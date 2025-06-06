#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;
in vec3 fragPosition;
// Input uniform values
//uniform sampler2D diffuse;
//uniform sampler2D normals;
//uniform vec4 colDiffuse;
// Output fragment color
out vec4 finalColor;
void main()
{
    vec3 light_pos = vec3(0.0, 0.0,0.0);
    vec3 delt = light_pos-fragPosition;
    float dist = delt.x*delt.x+delt.y*delt.y+delt.z*delt.z;
    dist /= 25.0;
    vec3 norm = fragNormal;
    float delta = norm.x*delt.x+norm.y*delt.y+norm.z*delt.z;
    delta /= dist;
    delta = log(delta);
    if (delta<0.1){
        delta = 0.1;
    }
    else if(delta>1.0){
        delta = 1.0;        
    }
    
    finalColor = delta *vec4(1,1,1,1);
    finalColor.a = 1;
    /*
    vec3 light_pos = vec3(-10.0, 0.0,0.0);
    vec3 delt = light_pos-position;
    float dist = delt.x*delt.x+delt.y*delt.y+delt.z*delt.z;
    vec4 norm = texture(normals,fragTexCoord);
    float delta = norm.x*delt.x+norm.y*delt.y+norm.z*delt.z;
    delta /= dist;
    delta*=5.0;
    if (delta<0.2){
        delta = 0.2;
    }
    else if(delta>1.0){
        delta = 1.0;
        
    }
    finalColor = delta * texture(diffuse, fragTexCoord);
    */
    /*
    float dx = fragTexCoord.x-0.5;
    float dy = fragTexCoord.y-0.5;
    finalColor = vec4(1.0, 0.8,0.8, 1.0);
    if (dx*dx+dy*dy>0.5){
        finalColor.a = 0.0;
    }*/
}
