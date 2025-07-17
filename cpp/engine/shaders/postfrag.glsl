#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;
in vec3 fragPosition;
// Input uniform values
uniform sampler2D inputTexture;
//uniform sampler2D normals;
//uniform vec4 colDiffuse;
// Output fragment color
out vec4 finalColor;
void main(){
    vec4 col =   texture(inputTexture, fragTexCoord);
    vec4 col0 = col;
    /*
    if(col.r<col0.r){col.r = col0.r;}
    if(col.g<col0.g){col.g = col0.g;}
    if(col.b<col0.b){col.b = col0.b;}*/
    float a = col.a; 
    float total = 1.0;
    int width =4;
    for(int i  = -width; i<width+1; i++){
        for(int j = -width; j<width+1; j++){
            if(i == 0 && j == 0){
                continue;
            }
            vec4 c2 = texture(inputTexture, fragTexCoord+vec2(i,j));
            float div = exp((i*i+j*j)/100.0);
            c2 /=div;
            total +=div;
            float thresh =0.0;
            if(c2.r >= thresh|| c2.g>=thresh || c2.b>= thresh){ 
                col += c2;
            }
        }
    }
    col /= total;
    //col/= (width+1)*(width+1);
    float ts =101000.00;
    finalColor =col; 
    finalColor.a = 1.0;
}
