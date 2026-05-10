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
const int direct_light_size = 1024;
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

vec4 array_access(int array, int x, int y) {
  int base_x = array % 16;
  int base_y = array / 16;
  float dx = float(x + base_x * 256 + 0.5) / (4096.);
  float dy = float(y + base_y * 256 + 0.5) / (4096.);
  vec2 point = vec2(dx, dy);
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
  x += float(direct_light_size) / 2.;
  z -= float(direct_light_size) / 2.;
  vec3 xyz = vec3(0.0, 512., 0.0);
  vec4 at = array_access(0, int(x), int(z));
  vec4 at2 = array_access(0, int(x + 2.5), int(z + 2.5));
  vec4 at3 = array_access(0, int(x + 2.5), int(z - 2.5));
  vec4 at4 = array_access(0, int(x - 2.5), int(z + 2.5));
  vec4 at5 = array_access(0, int(x - 2.5), int(z - 2.5));
  float v = (at.r + at.g * 256. + at.b * 256. * 256.);
  float v1 = (at2.r + at2.g * 256. + at2.b * 256. * 256.);
  float v2 = (at3.r + at3.g * 256. + at3.b * 256. * 256.);
  float v3 = (at4.r + at4.g * 256. + at4.b * 256. * 256.);
  float v4 = (at5.r + at5.g * 256. + at5.b * 256. * 256.);
  v = min(min(min(v1, v2), min(v3, v4)), v);
  if (v >= dot((fragPosition - xyz), directional_light_direction) - 190.) {
    float tmp = dot(-directional_light_direction, fragNormal);
    if (tmp < 0.0) {
      tmp = 0.0;
    }
    o += tmp * directional_light_color;
  }
  return o;
}

void main() {
  vec4 texelColor = texture(texture0, fragTexCoord);
  finalColor = texelColor;
  vec4 tmp = from_directional();
  tmp.r *= texelColor.r;
  tmp.g *= texelColor.g;
  tmp.b *= texelColor.b;
  for (int i = 0; i < light_count; i++) {
    tmp += contribution(i, texelColor);
  }

  tmp.r = pow(tmp.r, 1. / 1.1);
  tmp.g = pow(tmp.g, 1. / 1.1);
  tmp.b = pow(tmp.b, 1. / 1.1);
  tmp.a = texelColor.a;
  finalColor = tmp;
}
