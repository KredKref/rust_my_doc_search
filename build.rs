extern crate embed_resource;
fn main() {
    //设置图标
    //C:\Program Files (x86)\Windows Kits\10\bin\10.0.18362.0\x64 添加到环境变量path中,然后我们就可以用rc.exe
    
    embed_resource::compile("./icon.rc");
}