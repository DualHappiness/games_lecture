### 直接光照着色
主要修改了 EvalDiffuse 和 EvalDirectLight
```GLSL
// ssrFragment.glsl
vec3 EvalDiffuse(vec3 _wi, vec3 _wo, vec2 uv) {
  return GetGBufferDiffuse(uv) * INV_PI;
}

vec3 EvalDirectionalLight(vec2 uv) {
  float visibility = GetGBufferuShadow(uv);
  vec3 normal = GetGBufferNormalWorld(uv);
  return visibility * max(dot(uLightDir, normal), 0.0) * uLightRadiance;
}
```

### SSR求交
实现了mipmap 利用创建多个framebuffer然后缩减尺寸渲染 每次保存最小值实现了加速查找的mipmap
```GLSL
// mipmapVertex.glsl
attribute vec2 aCoord;

void main() {
    gl_Position = vec4(aCoord, 0.0, 1.0);
}

// mipmapFragment.glsl
#ifdef GL_ES
#extension GL_EXT_draw_buffers: enable
precision highp float;
#endif

uniform vec4 uViewport;
uniform sampler2D uGDepth;

void main() {
    vec2 uv = (gl_FragCoord.xy - uViewport.xy) / uViewport.zw;
    float invWidth = 1.0 / uViewport.z;
    float invHeight = 1.0 / uViewport.w;
    float d1 = texture2D(uGDepth, uv + vec2(invWidth, invHeight)).x;
    float d2 = texture2D(uGDepth, uv + vec2(-invWidth, invHeight)).x;
    float d3 = texture2D(uGDepth, uv + vec2(invWidth, -invHeight)).x;
    float d4 = texture2D(uGDepth, uv + vec2(-invWidth, -invHeight)).x;

    float v = min(min(d1, d2), min(d3, d4));
    gl_FragData[0] = vec4(v);
    // gl_FragData[0] = vec4(uv, 0.0, 0.0);
}
```
```javascript
// WebGLRenderer.js
const program = this.mipmapShader.program;
for (let level = 0; level < mipmapLevel; level++) {
    const width = window.screen.width >> (level + 1);
    const height = window.screen.height >> (level + 1);
    const fbo = this.camera.mipmapFbos[level];

    gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
    gl.viewport(0, 0, window.screen.width, window.screen.height);
    gl.useProgram(program.glShaderProgram);

    gl.uniform4fv(program.uniforms.uViewport, [0, 0, width, height]);
    gl.activeTexture(gl.TEXTURE0 + 0);
    gl.bindTexture(gl.TEXTURE_2D, textures[level]);
    gl.uniform1i(program.uniforms.uGDepth, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.#points);
    gl.vertexAttribPointer(program.attribs.aCoord, 2, gl.FLOAT, false, 0, 0);
    gl.enableVertexAttribArray(program.attribs.aCoord);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.#indices);
    gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);

    gl.bindFramebuffer(gl.FRAMEBUFFER, null);
}
```
然后求交过程就很简单了
```GLSL
float GetGBufferDepthLod(vec2 uv, int level) {
  float depth = 0.0;
  if(level == 0) {
    depth = GetGBufferDepth(uv);
  } else if(level == 1) {
    depth = texture2D(uGDepth1, uv).x;
  } else if(level == 2) {
    depth = texture2D(uGDepth2, uv).x;
  } else if(level == 3) {
    depth = texture2D(uGDepth3, uv).x;
  } else {
    depth = 10000.0;
  }

  return depth;
}

float StepLevel(float step, int level) {
  return step * pow(2.0, float(level));
}

bool RayMarch(vec3 ori, vec3 dir, out vec3 hitPos) {
  int level = 3;
  float step = 0.01;
  float dis = StepLevel(step, level);
  hitPos = ori;
  for(int i = 0; i < 1000; i++) {
    hitPos = ori + dir * dis;
    float depth = GetGBufferDepthLod(GetScreenCoordinate(hitPos), level);
    if(depth + BIAS > GetDepth(hitPos)) {
      if(level < 3) {
        level++;
      }
    } else {
      dis -= StepLevel(step, level);
      level--;
      if(level < 0) {
        break;
      }
    }
    dis += StepLevel(step, level);
  }

  return level < 0;
}
```

### 场景间接光照着色
利用 raymarch 和随机产生的光线 去寻找交点，然后计算间接光的贡献
```GLSL
// ssrFragment.glsl
vec3 diffuse = EvalDiffuse(uLightDir, wo, uv0);
vec3 t, b;
LocalBasis(normal, t, b);
t = normalize(t);
b = normalize(b);
mat3 localToWorld = mat3(t, b, normal);
for(int i = 0; i < SAMPLE_NUM; i++) {
float pdf = 0.0;
vec3 local_dir = SampleHemisphereUniform(s, pdf);
vec3 dir = normalize(localToWorld * local_dir);
vec3 hitPos;
if(RayMarch(pos, dir, hitPos)) {
    vec2 uv1 = GetScreenCoordinate(hitPos);
    vec3 LSample = diffuse / pdf * EvalDiffuse(uLightDir, -dir, uv1) * EvalDirectionalLight(uv1);
    LIndirect += LSample;
}
}
LIndirect /= float(SAMPLE_NUM);
```