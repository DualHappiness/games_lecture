# Denoise

## 实现单帧降噪
`Denoiser::Filter` 完成联合双边滤波算法

## 实现两帧间投影
`Denoiser::Reprojection` 利用保存的matrix信息完成当前像素对上帧像素的投影

## 实现两帧间累积
`Denoiser::TemporalAccumulation` 用7x7的滤波核实现outlier removal 根据投影结果做插值

## 实现了 a-trous wavelet 加速
`Denoiser::Filter  line 88` 用多趟小核代替大核实现加速

## 其他修改
修改了 sigmaColor 为 4.6  要不然pinkroom亮点太多了