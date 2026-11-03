"""Package native release outputs. Run from any directory with platform and output path."""
import argparse
from pathlib import Path
import zipfile
p=argparse.ArgumentParser()
p.add_argument('platform',choices=['linux-x64','windows-x64'])
p.add_argument('output',type=Path)
a=p.parse_args()
root=Path(__file__).resolve().parents[1]
a.output.mkdir(parents=True,exist_ok=True)
names=['garlic-proxy','garlic-dns','garlic-infinite-storage','garlic-internal-network'] if a.platform=='linux-x64' else ['garlic-proxy','garlic-internal-network']
with zipfile.ZipFile(a.output/f'garlic-v0.3.0-{a.platform}.zip','w',zipfile.ZIP_DEFLATED) as z:
    for name in names:
        filename=name+('.exe' if a.platform=='windows-x64' else '')
        z.write(root/'target/release'/filename,filename)
    for path in [root/'README.md',*root.joinpath('docs').rglob('*.md'),*root.joinpath('configs').glob('*')]:
        if path.is_file(): z.write(path,str(path.relative_to(root)))
