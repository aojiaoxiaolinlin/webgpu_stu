use render_backend::backend::get_device_and_queue;

fn main() -> anyhow::Result<()> {
    let (device, queue) = futures::executor::block_on(get_device_and_queue())?;

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("compute_pipeline"),
        layout: todo!(),
        module: todo!(),
        entry_point: todo!(),
        compilation_options: todo!(),
        cache: todo!(),
    });
    Ok(())
}
