pub fn module(
    compiler: &mut shaderc::Compiler,
    device: &wgpu::Device,
    shader_kind: shaderc::ShaderKind,
    src_str: &str,
    src_filename: &str,
    label: &str,
) -> Result<wgpu::ShaderModule, shaderc::Error> {
    let spirv = compiler
        .compile_into_spirv(src_str, shader_kind, src_filename, "main", None)
        .unwrap();

    let data = wgpu::util::make_spirv(spirv.as_binary_u8());

    let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: data,
        flags: wgpu::ShaderFlags::default(),
    });

    Ok(module)
}

pub fn fragment_module(
    compiler: &mut shaderc::Compiler,
    device: &wgpu::Device,
    src_str: &str,
    src_filename: &str,
    label: &str,
) -> Result<wgpu::ShaderModule, shaderc::Error> {
    module(
        compiler,
        device,
        shaderc::ShaderKind::Fragment,
        src_str,
        src_filename,
        label,
    )
}

pub fn vertex_module(
    compiler: &mut shaderc::Compiler,
    device: &wgpu::Device,
    src_str: &str,
    src_filename: &str,
    label: &str,
) -> Result<wgpu::ShaderModule, shaderc::Error> {
    module(
        compiler,
        device,
        shaderc::ShaderKind::Vertex,
        src_str,
        src_filename,
        label,
    )
}
