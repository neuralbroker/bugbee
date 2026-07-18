export const meshGradientVertex = /* glsl */ `
  varying vec2 vUv;
  varying float vDisplacement;
  uniform float uTime;
  uniform vec2 uMouse;

  // Simplex 3D noise
  vec3 mod289(vec3 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec4 mod289(vec4 x) { return x - floor(x * (1.0 / 289.0)) * 289.0; }
  vec4 permute(vec4 x) { return mod289(((x * 34.0) + 1.0) * x); }
  vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }

  float snoise(vec3 v) {
    const vec2 C = vec2(1.0 / 6.0, 1.0 / 3.0);
    const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);
    vec3 i = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);
    vec3 x1 = x0 - i1 + C.xxx;
    vec3 x2 = x0 - i2 + C.yyy;
    vec3 x3 = x0 - D.yyy;
    i = mod289(i);
    vec4 p = permute(permute(permute(
      i.z + vec4(0.0, i1.z, i2.z, 1.0))
      + i.y + vec4(0.0, i1.y, i2.y, 1.0))
      + i.x + vec4(0.0, i1.x, i2.x, 1.0));
    float n_ = 0.142857142857;
    vec3 ns = n_ * D.wyz - D.xzx;
    vec4 j = p - 49.0 * floor(p * ns.z * ns.z);
    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_);
    vec4 x = x_ * ns.x + ns.yyyy;
    vec4 y = y_ * ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);
    vec4 b0 = vec4(x.xy, y.xy);
    vec4 b1 = vec4(x.zw, y.zw);
    vec4 s0 = floor(b0) * 2.0 + 1.0;
    vec4 s1 = floor(b1) * 2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));
    vec4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    vec4 a1 = b1.xzyw + s1.xzyw * sh.zzww;
    vec3 p0 = vec3(a0.xy, h.x);
    vec3 p1 = vec3(a0.zw, h.y);
    vec3 p2 = vec3(a1.xy, h.z);
    vec3 p3 = vec3(a1.zw, h.w);
    vec4 norm = taylorInvSqrt(vec4(dot(p0, p0), dot(p1, p1), dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
    vec4 m = max(0.6 - vec4(dot(x0, x0), dot(x1, x1), dot(x2, x2), dot(x3, x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m * m, vec4(dot(p0, x0), dot(p1, x1), dot(p2, x2), dot(p3, x3)));
  }

  void main() {
    vUv = uv;
    vec3 pos = position;
    float mouseInfluence = length(uv - (uMouse * 0.5 + 0.5)) * 2.0;
    float n = snoise(vec3(pos.x * 0.8 + uTime * 0.15, pos.y * 0.8, uTime * 0.1));
    float displacement = n * 0.35 * (1.0 - mouseInfluence * 0.15);
    pos.z += displacement;
    vDisplacement = displacement;
    gl_Position = projectionMatrix * modelViewMatrix * vec4(pos, 1.0);
  }
`;

export const meshGradientFragment = /* glsl */ `
  varying vec2 vUv;
  varying float vDisplacement;
  uniform float uTime;
  uniform vec2 uMouse;

  void main() {
    // Deep purple → amber → void black
    vec3 purple = vec3(0.102, 0.043, 0.180); // #1a0b2e
    vec3 amber = vec3(0.961, 0.620, 0.043);  // #f59e0b
    vec3 voidBlack = vec3(0.020, 0.020, 0.031); // #050508
    vec3 cyan = vec3(0.133, 0.827, 0.933); // accent

    float t = uTime * 0.12;
    float wave = sin(vUv.x * 3.0 + t) * cos(vUv.y * 2.5 - t * 0.7);
    float n = vDisplacement * 2.0 + 0.5;

    vec2 mouseUv = uMouse * 0.5 + 0.5;
    float mouseDist = length(vUv - mouseUv);

    vec3 col = mix(voidBlack, purple, smoothstep(0.0, 0.7, vUv.y + wave * 0.15));
    col = mix(col, amber * 0.55, smoothstep(0.2, 0.9, n) * 0.55);
    col = mix(col, cyan * 0.25, smoothstep(0.6, 1.0, vUv.x + sin(t) * 0.1) * 0.35);
    col += amber * 0.12 * (1.0 - smoothstep(0.0, 0.45, mouseDist));

    // Vignette
    float vig = smoothstep(1.2, 0.3, length(vUv - 0.5) * 1.4);
    col *= vig * 0.85 + 0.15;

    gl_FragColor = vec4(col, 1.0);
  }
`;
