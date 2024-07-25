# WebGPU 学习

绑定组（`BindGroup`）描述了一组资源以及如何通过着色器访问它们。我们先来创建一个绑定组布局（`BindGroupLayout`）：

> 创建顺序`BindGroupLayout` -> `BindGroup` -> `PipelineLayout` -> `RenderPipeline`
