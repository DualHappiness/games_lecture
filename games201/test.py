import taichi as ti
ti.init(arch=ti.gpu)

a = ti.var(dt=ti.f32, shape=())
a[None] = 1
print(a[None])
