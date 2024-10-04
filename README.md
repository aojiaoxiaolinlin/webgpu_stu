# WebGPU 学习

绑定组（`BindGroup`）描述了一组资源以及如何通过着色器访问它们。我们先来创建一个绑定组布局（`BindGroupLayout`）：

> 创建顺序`BindGroupLayout` -> `BindGroup` -> `PipelineLayout` -> `RenderPipeline`

## 模型矩阵

$$
平移矩阵 \times 缩放矩阵
$$

原始位置为(2,0,0)

1. 平移矩阵

   沿 $x$ 轴平移两个单位

   $$
   \begin{bmatrix}
        1 & 0 & 0 & 2 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
       \times
    \begin{bmatrix}
        2 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
       =
    \begin{bmatrix}
        4 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
   $$

2. 缩放矩阵($x$轴方向缩放 10 倍)

   $$
   \begin{bmatrix}
        10 & 0 & 0 & 0 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
        \times
    \begin{bmatrix}
        2 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
       =
    \begin{bmatrix}
        20 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
   $$

**模型矩阵:** 先平移后缩放，先平移需要放后面(矩阵乘法不满足交换律)

$$
\begin{bmatrix}
        1 & 0 & 0 & 2 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
       \times
    \begin{bmatrix}
        10 & 0 & 0 & 0 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
        =
    \begin{bmatrix}
        10 & 0 & 0 & 20 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
$$

最终位置为

$$
\begin{bmatrix}
        10 & 0 & 0 & 20 \\
        0 & 1 & 0 & 0 \\
        0 & 0 & 1 & 0 \\
        0 & 0 & 0 & 1
    \end{bmatrix}
       \times
    \begin{bmatrix}
        2 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
        =
    \begin{bmatrix}
        40 \\
        0 \\
        0 \\
        1 \\
    \end{bmatrix}
$$
