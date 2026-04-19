#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragPosition;
in vec3 fragNormal;

// Input uniform values
uniform sampler2D texture0;
uniform sampler2D texture1;

uniform vec4 colDiffuse;

// Output fragment color
out vec4 finalColor;

// NOTE: Add your custom variables here
uniform vec3 light_positions[16];
uniform vec4 light_colors[16];
uniform int light_count;

uniform vec4 directional_light_color;
uniform vec3 camera_position;
uniform vec4 ambient;
uniform vec3 directional_light_direction;
const int direct_light_size = 256;
int offset_from_delta(vec3 delta) {
  //    let theta = v.y.atan2(v.x);
  // let r = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
  // let phi = (v.z / r).acos();
  //(r, theta, phi)
  float r = length(delta);
  float theta = atan(delta.y, delta.x) / 6.28;
  float phi = acos(delta.z / r) / 6.28;
  return int(16 * 16 * phi) + int(16 * theta) +
         direct_light_size * direct_light_size;
}

vec4 array_access(int at) {
  int dx = at % 4096;
  int dy = at / 4096;
  float x = float(dx) / (4096.);
  float y = float(dy) / (4096.);
  vec2 point = vec2(x, y);
  return texture(texture1, point);
}

vec4 contribution(int light_idx, vec4 texel_color) {
  vec4 ot = vec4(0.0, 0.0, 0.0, 0.0);
  vec3 delta = light_positions[light_idx] - fragPosition;
  float len = length(delta);
  if (len > 0.0) {
    vec3 dt = normalize(delta);
    ot = light_colors[light_idx] * (dot(dt, -fragNormal)) / (len) * 10;
  }
  ot.r *= texel_color.r;
  ot.g *= texel_color.g;
  ot.b *= texel_color.b;
  ot.a *= texel_color.a;
  return ot;
}

vec4 from_directional() {
  vec4 o = ambient;

  float x = fragPosition.x;
  float z = fragPosition.z;
  x += float(direct_light_size / 2);
  z += float(direct_light_size / 2);
  int pos = int((z * float(direct_light_size) + x));
  if (pos >= 256 * 256) {
    o.r = 1.0;
    o.g = 0.0;
    o.b = 0.0;
    return o;
  }
  vec3 xyz = vec3(fragPosition.x, 128., fragPosition.z);
  vec4 at = array_access(pos);
  if (true) {
    o.r = at.b;
    o.g = 0.0;
    o.b = 0.0;
    // o.g = x / 256.;
    // o.b = z / 256.;
    return o;
  }
  if ((at.r) * 256. >= length(xyz - fragPosition)) {
    o +=
        dot(directional_light_direction, -fragNormal) * directional_light_color;
  }
  return o;
}

void main() {
  vec4 texelColor = texture(texture0, fragTexCoord);
  finalColor = texelColor;
  vec4 tmp = from_directional();
  // tmp.r *= texelColor.r;
  // tmp.g *= texelColor.g;
  // tmp.b *= texelColor.b;
  if (false) {
    for (int i = 0; i < light_count; i++) {
      tmp += contribution(i, texelColor);
    }
  }
  // tmp.r = pow(tmp.r, 1. / 1.1);
  // tmp.g = pow(tmp.g, 1. / 1.1);
  // tmp.b = pow(tmp.b, 1. / 1.1);
  tmp.a = texelColor.a;
  finalColor = tmp;
}
