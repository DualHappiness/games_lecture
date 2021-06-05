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