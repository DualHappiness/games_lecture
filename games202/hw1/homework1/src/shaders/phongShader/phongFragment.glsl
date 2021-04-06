#ifdef GL_ES
precision mediump float;
#endif
#define LIGHT_MAX 2

// Phong related variables
uniform sampler2D uSampler;
uniform vec3 uKd;
uniform vec3 uKs;
uniform vec3 uLightPos[LIGHT_MAX];
// uniform vec3 uSubLightPos;
uniform vec3 uCameraPos;
uniform vec3 uLightIntensity[LIGHT_MAX];

varying highp vec2 vTextureCoord;
varying highp vec3 vFragPos;
varying highp vec3 vNormal;

// Shadow map related variables
#define NUM_SAMPLES 50
#define BLOCKER_SEARCH_NUM_SAMPLES NUM_SAMPLES
#define PCF_NUM_SAMPLES NUM_SAMPLES
#define NUM_RINGS 10

#define EPS 1e-3
#define PI 3.141592653589793
#define PI2 6.283185307179586

#define LIGHT_WIDTH 0.01

uniform sampler2D uShadowMap[LIGHT_MAX];

varying vec4 vPositionFromLight[LIGHT_MAX];

highp float rand_1to1(highp float x) { 
  // -1 -1
  return fract(sin(x) * 10000.0);
}

highp float rand_2to1(vec2 uv) { 
  // 0 - 1
  const highp float a = 12.9898, b = 78.233, c = 43758.5453;
  highp float dt = dot(uv.xy, vec2(a, b)), sn = mod(dt, PI);
  return fract(sin(sn) * c);
}

float unpack(vec4 rgbaDepth) {
  const vec4 bitShift = vec4(1.0, 1.0 / 256.0, 1.0 / (256.0 * 256.0), 1.0 / (256.0 * 256.0 * 256.0));
  return dot(rgbaDepth, bitShift);
}

vec2 poissonDisk[NUM_SAMPLES];

void poissonDiskSamples(const in vec2 randomSeed) {

  float ANGLE_STEP = PI2 * float(NUM_RINGS) / float(NUM_SAMPLES);
  float INV_NUM_SAMPLES = 1.0 / float(NUM_SAMPLES);

  float angle = rand_2to1(randomSeed) * PI2;
  float radius = INV_NUM_SAMPLES;
  float radiusStep = radius;

  for(int i = 0; i < NUM_SAMPLES; i++) {
    poissonDisk[i] = vec2(cos(angle), sin(angle)) * pow(radius, 0.75);
    radius += radiusStep;
    angle += ANGLE_STEP;
  }
}

void uniformDiskSamples(const in vec2 randomSeed) {

  float randNum = rand_2to1(randomSeed);
  float sampleX = rand_1to1(randNum);
  float sampleY = rand_1to1(sampleX);

  float angle = sampleX * PI2;
  float radius = sqrt(sampleY);

  for(int i = 0; i < NUM_SAMPLES; i++) {
    poissonDisk[i] = vec2(radius * cos(angle), radius * sin(angle));

    sampleX = rand_1to1(sampleY);
    sampleY = rand_1to1(sampleX);

    angle = sampleX * PI2;
    radius = sqrt(sampleY);
  }
}

float findBlocker(sampler2D shadowMap, vec2 uv, float zReceiver) {
  // uniformDiskSamples(uv);
  poissonDiskSamples(uv);
  float filter_size = clamp(0.2 * (zReceiver - 0.1) / zReceiver / 5.0, 0.0, 0.02);
  float sum = zReceiver;
  float blockers = 1.0;
  for(int i = 0; i < BLOCKER_SEARCH_NUM_SAMPLES; i++) {
    float depth = unpack(texture2D(shadowMap, uv + filter_size * poissonDisk[i]));
    float block = max(sign(zReceiver - depth - EPS), 0.0);
    sum += mix(0.0, depth, block);
    blockers += block;
  }
  return sum / blockers;
}

float PCF(sampler2D shadowMap, vec4 coords) {
  // uniformDiskSamples(coords.xy);
  poissonDiskSamples(coords.xy);
  float v = 0.0;
  float filter_size = coords.w;
  for(int i = 0; i < PCF_NUM_SAMPLES; i++) {
    vec2 uv = coords.xy + filter_size * poissonDisk[i];
    float shadowDepth = unpack(texture2D(shadowMap, uv));
    v += max(sign(shadowDepth - coords.z + EPS), 0.0);
  }
  return (v + 1.0) / (float(PCF_NUM_SAMPLES) + 1.0);
}

float PCSS(sampler2D shadowMap, vec4 coords) {

  // STEP 1: avgblocker depth
  float avg = findBlocker(shadowMap, coords.xy, coords.z);

  // STEP 2: penumbra size
  float w = max((coords.z - avg) * float(LIGHT_WIDTH) / avg, 0.0);

  // STEP 3: filtering
  return PCF(shadowMap, vec4(coords.xyz, w));
}

float useShadowMap(sampler2D shadowMap, vec4 shadowCoord) {
  float shadowDepth = unpack(texture2D(shadowMap, shadowCoord.xy));
  return min(sign(shadowDepth - shadowCoord.z + EPS), 1.0);
}

vec3 blinnPhong(vec3 lightPos, vec3 lightIntensity) {
  vec3 color = texture2D(uSampler, vTextureCoord).rgb;
  color = pow(color, vec3(2.2));

  vec3 ambient = 0.05 * color;

  vec3 lightDir = normalize(lightPos);
  vec3 normal = normalize(vNormal);
  float diff = max(dot(lightDir, normal), 0.0);
  vec3 light_atten_coff = lightIntensity / pow(length(lightPos - vFragPos), 2.0);
  vec3 diffuse = diff * light_atten_coff * color;

  vec3 viewDir = normalize(uCameraPos - vFragPos);
  vec3 halfDir = normalize((lightDir + viewDir));
  float spec = pow(max(dot(halfDir, normal), 0.0), 32.0);
  vec3 specular = uKs * light_atten_coff * spec;

  vec3 radiance = (ambient + diffuse + specular);
  vec3 phongColor = pow(radiance, vec3(1.0 / 2.2));
  return phongColor;
}

void main(void) {

  float visibility = 1.0;
  vec3 color = vec3(0.0);
  for(int i = 0; i < LIGHT_MAX; i++) {
    // map [-1,1] to [0, 1]

    vec3 shadowCoord = vPositionFromLight[i].xyz / 2.0 + 0.5;
    // visibility = useShadowMap(uShadowMap[i], vec4(shadowCoord, 1.0));
    // visibility = PCF(uShadowMap[i], vec4(shadowCoord, 0.005));
    visibility = PCSS(uShadowMap[i], vec4(shadowCoord, 1.0));
    color += blinnPhong(uLightPos[i], uLightIntensity[i]) * visibility;
  }

  // gl_FragColor = vec4(1.0 - visibility);

  gl_FragColor = vec4(color, 1.0);

}