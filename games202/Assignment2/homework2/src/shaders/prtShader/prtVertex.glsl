attribute vec3 aVertexPosition;
attribute vec2 aTextureCoord;
attribute mat3 aPrecomputeLT;

uniform sampler2D uSampler;
// uniform vec3 uPrecomputeL[9];
uniform mat4 uModelMatrix;
uniform mat4 uViewMatrix;
uniform mat4 uProjectionMatrix;

varying highp vec3 vColor;

void main(void) {

  vec3 color = vec3(0.0);
  for(int i = 0; i < 3; i++) {
    for(int j = 0; j < 3; j++) {
      // vec3 sh = vec3(uPrecomputeL[3 * j][i], uPrecomputeL[3 * j + 1][i], uPrecomputeL[3 * j + 2][i]);
      vec3 L = vec3(1.0);
      // color[i] += dot(L, aPrecomputeLT[j]);
    }
  }
  gl_Position = uProjectionMatrix * uViewMatrix * uModelMatrix *
    vec4(aVertexPosition, 1.0);

  vColor = color * texture2D(uSampler, aTextureCoord).rgb;
}