"""Check the packaged Android loader using apktool-decoded files, pyelftools and Unicorn.

JNI and libdl calls are modelled. The binder and the core ABI/version functions execute
from the released ELF files. RegisterNatives must match the declarations in classes.dex.
"""
import argparse
import re
from pathlib import Path

from elftools.elf.elffile import ELFFile
from unicorn import Uc, UC_ARCH_ARM, UC_ARCH_ARM64, UC_MODE_ARM, UC_HOOK_CODE
from unicorn import arm_const as arm, arm64_const as a64


def run(directory, abi, required, methods):
    wide = abi == "arm64-v8a"
    word = 8 if wide else 4
    vm = Uc(UC_ARCH_ARM64 if wide else UC_ARCH_ARM, UC_MODE_ARM)
    vm.mem_map(0x10000, 0x4000000)
    args = [(a64.UC_ARM64_REG_X0 if wide else arm.UC_ARM_REG_R0) + i for i in range(4)]
    def put(p, n):
        vm.mem_write(p, n.to_bytes(word, "little"))
    def read(p):
        return int.from_bytes(vm.mem_read(p, word), "little")
    def string(p):
        return bytes(vm.mem_read(p, 512)).split(b"\0", 1)[0].decode()
    hooks = {}
    def stub(name):
        p = 0x3000000 + 4 * len(hooks)
        hooks[p] = name
        vm.mem_write(p, bytes.fromhex("c0035fd6" if wide else "1eff2fe1"))
        return p
    def load(name, base, relocate):
        with (directory / "lib" / abi / name).open("rb") as f:
            elf = ELFFile(f)
            for seg in elf.iter_segments():
                if seg["p_type"] == "PT_LOAD":
                    vm.mem_write(base + seg["p_vaddr"], seg.data())
            syms = elf.get_section_by_name(".dynsym")
            for sec in elf.iter_sections():
                if sec.name not in (".rela.dyn", ".rela.plt", ".rel.dyn", ".rel.plt"):
                    continue
                for rel in sec.iter_relocations():
                    p = base + rel["r_offset"]
                    if rel["r_info_type"] == (1027 if wide else 23):
                        put(p, base + (rel["r_addend"] if rel.is_RELA() else read(p)))
                    elif relocate:
                        put(p, stub(syms.get_symbol(rel["r_info_sym"]).name))
            return {s.name:base+s["st_value"] for s in syms.iter_symbols() if s["st_value"]}
    core = load("libgarlic_internal_network.so", 0x1000000, False)
    bind = load("libgarlic_bind.so", 0x100000, True)["Java_org_garlic_client_CoreLoader_bind"]
    env, table, version, stack, end = 0x3100000, 0x3110000, 0x3120000, 0x3200000, 0x3210000
    put(env,table)
    vm.mem_write(version, required.encode()+b"\0")
    for i, n in {6:"class",17:"clear",113:"method",169:"utf",170:"release",215:"register",228:"exception"}.items():
        put(table+i*word,stub(n))
    registered=set()
    def dispatch(cpu, addr, size, data):
        name=hooks[addr];a,b,c,d=[cpu.reg_read(r) for r in args];result=0
        if name=="utf": result=b
        elif name=="dlopen": result=1
        elif name=="dlsym": result=core.get(string(b),0)
        elif name=="strlen": result=len(string(a))
        elif name=="memcmp": result=int(cpu.mem_read(a,c)!=cpu.mem_read(b,c))
        elif name in ("memcpy","__aeabi_memcpy","__aeabi_memcpy4","__aeabi_memcpy8"): cpu.mem_write(a,bytes(cpu.mem_read(b,c)));result=a
        elif name=="class": result=int(string(b)=="org/garlic/client/GarlicBridge")
        elif name=="method": result=int((string(c),string(d)) in methods)
        elif name=="register":
            for i in range(d):
                p=c+3*i*word
                m,sig=string(read(p)),string(read(p+word))
                assert (m,sig) in methods and core["Java_org_garlic_client_GarlicBridge_"+m]==read(p+2*word)
                registered.add((m,sig))
        elif name not in ("exception","clear","dlclose","release"): raise AssertionError(name)
        cpu.reg_write(args[0],result)
    vm.hook_add(UC_HOOK_CODE,dispatch,begin=0x3000000,end=0x300ffff)
    for reg,arg in zip(args,(env,0,0x3130000,version)):vm.reg_write(reg,arg)
    vm.reg_write(a64.UC_ARM64_REG_X30 if wide else arm.UC_ARM_REG_LR,end)
    vm.reg_write(a64.UC_ARM64_REG_SP if wide else arm.UC_ARM_REG_SP,stack)
    vm.emu_start(bind,end,count=250000)
    assert vm.reg_read(a64.UC_ARM64_REG_PC if wide else arm.UC_ARM_REG_PC)==end
    return vm.reg_read(args[0]),len(registered)


if __name__=="__main__":
    parser=argparse.ArgumentParser()
    parser.add_argument("decoded",type=Path)
    parser.add_argument("app_version")
    parser.add_argument("--expect-rejection",action="store_true")
    opt=parser.parse_args()
    classes=opt.decoded/"smali/org/garlic/client"
    init=(classes/"CoreLoader.smali").read_text().split(".method static initialize(",1)[1].split(".end method",1)[0]
    assert "CoreLoader;->bind(Ljava/lang/String;Ljava/lang/String;)Z" in init
    match=re.search(r'const-string v0, "([0-9.]+)"', init)
    if match: required=match.group(1)
    else:
        assert "PackageInfo;->versionName:" in init
        required=opt.app_version
    methods=set(re.findall(r"\.method public static native (\w+)(\(.*)",(classes/"GarlicBridge.smali").read_text()))
    assert methods
    # The JNI stubs below cannot detect missing platform libraries at dlopen.
    for library in (opt.decoded / "lib").glob("*/libgarlic_bind.so"):
        with library.open("rb") as f:
            elf = ELFFile(f)
            dependencies = {tag.needed for tag in elf.get_section_by_name(".dynamic").iter_tags("DT_NEEDED")}
            assert {"libc.so", "libdl.so"} <= dependencies, (library, dependencies)
    for abi in ("arm64-v8a","armeabi-v7a"):
        ok,count=run(opt.decoded,abi,required,methods)
        print(abi, "bind",ok,"registered",count)
        assert ok==int(not opt.expect_rejection)
        if ok:
            assert count==len(methods)
            assert run(opt.decoded,abi,"incompatible",methods)==(0,0)
