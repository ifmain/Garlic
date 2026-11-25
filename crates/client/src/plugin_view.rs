use release_cache::canvas::Canvas;
use std::sync::Mutex;
static VIEW: Mutex<Option<Canvas>>=Mutex::new(None);

pub(crate) fn open(code:&str)->bool {
    let mut view=VIEW.lock().unwrap();
    *view=Canvas::new(code).ok();
    view.is_some()
}
pub(crate) fn tick(x:i32,y:i32,down:i32,ms:i32)->Option<Vec<i32>>{
    let mut view=VIEW.lock().unwrap();
    let result=view.as_mut()?.frame(x,y,down,ms).ok();
    if result.is_none(){*view=None;}
    result
}
#[no_mangle]
pub extern "C" fn garlic_view_close(){*VIEW.lock().unwrap()=None;}

#[cfg(windows)]
#[no_mangle]
pub unsafe extern "C" fn garlic_view_open(ptr:*const u8,len:usize)->i32{
    if ptr.is_null()||len>8192{return 0}
    std::str::from_utf8(std::slice::from_raw_parts(ptr,len)).ok().is_some_and(open) as i32
}
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "C" fn garlic_view_tick(x:i32,y:i32,down:i32,ms:i32,out:*mut i32,cap:usize)->i32{
    if out.is_null(){return -1}
    match tick(x,y,down,ms){
        Some(v) if v.len()<=cap=>{std::ptr::copy_nonoverlapping(v.as_ptr(),out,v.len());v.len() as i32}
        _=>-1
    }
}
