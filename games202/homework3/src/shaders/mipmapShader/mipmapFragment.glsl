#ifdef GL_ES
#extension GL_EXT_draw_buffers: enable
precision highp float;
#endif

uniform float uInvWidth;
uniform float uInvHeight;
// uniform sampler2D uGDepth;

void main() {
    vec2 uv = gl_FragCoord.xy * vec2(uInvHeight, uInvWidth);
    // float d1 = texture2D(uGDepth, uv + vec2(uInvWidth, uInvHeight)).x;
    // float d2 = texture2D(uGDepth, uv + vec2(-uInvWidth, uInvHeight)).x;
    // float d3 = texture2D(uGDepth, uv + vec2(uInvWidth, -uInvHeight)).x;
    // float d4 = texture2D(uGDepth, uv + vec2(-uInvWidth, -uInvHeight)).x;

    // gl_FragColor = vec4(uv, 1.0, 1.0);
    // gl_FragColor = vec4(texture2D(uGDepth, uv).xxx, 1.0);
    // float v = min(min(d1, d2), min(d3, d4));
    gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}