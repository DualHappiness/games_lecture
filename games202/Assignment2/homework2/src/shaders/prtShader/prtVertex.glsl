attribute mat3 aPrecomputeLT;
attribute vec3 aVertexPosition;
attribute vec3 aNormalPosition;
attribute vec2 aTextureCoord;

uniform sampler2D uSampler;
uniform mat3 uPrecomputeL[3];
uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;
uniform float uDiffuse;
#define PI 3.1415926533;

varying highp vec3 vColor;

void main(void) {

  vec3 color = vec3(0);
  for(int i = 0; i < 3; i++) {
    for(int j = 0; j < 3; j++) {
      // vec3 L = vec3(uPrecomputeL[3 * j][i], uPrecomputeL[3 * j + 1][i], uPrecomputeL[3 * j + 2][i]);
      vec3 L = uPrecomputeL[i][j];
      color[i] += dot(L, aPrecomputeLT[j]);
    }
  }
  // color = vec3(1.0);
  gl_Position = uProjectionMatrix * uViewMatrix * uModelMatrix *
    vec4(aVertexPosition, 1.0);

  // vColor = color * texture2D(uSampler, aTextureCoord).rgb;
  // 干脆不除pi了 直接控制还方便
  vColor = color * uDiffuse; // / PI;
}