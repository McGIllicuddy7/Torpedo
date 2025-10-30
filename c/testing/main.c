#include <tgmath.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <assert.h>
typedef struct {
	double x;
	double y;
	double z;
}Vec3;
Vec3 Vec3_add(Vec3 a, Vec3 b){
	return (Vec3){a.x+b.x, a.y+b.y, a.z+b.z};
}
Vec3 Vec3_sub(Vec3 a, Vec3 b){
	return (Vec3){a.x-b.x, a.y-b.y, a.z-b.z};
}
Vec3 Vec3_scale(Vec3 a,double s){
	return (Vec3){a.x*s, a.y*s, a.z*s};
}
double Vec3_len(Vec3 a){
	return sqrt(a.x*a.x+a.y*a.y+a.z*a.z);
}
double Vec3_dist(Vec3 a, Vec3 b){
	return Vec3_len(Vec3_sub(a,b));
}
double Vec3_dot(Vec3 a, Vec3 b){
	return a.x*b.x+a.y*b.y+a.z*b.z;
}
Vec3 Vec3_norm(Vec3 a){
	return Vec3_scale(a, 1./Vec3_len(a));
}
Vec3 next_impulse(Vec3 pos, Vec3 end, Vec3 vel,Vec3 des_vel, double acc){
	//A(t) = at+ b
	//V(t) = 1/2at^2+ bt + v_0
	//X(t) = 1/6at^3 + 1/2bt^2 + v_0t + x_0
	//X(0) =  x_0
	//X(T) = x_1 = 1/6at^3 + 1/2bt^2 + v_0t + x_0
	//V(T) =  0 = 1/2aT^2 + bT + v_0
	//T = -b + \sqrt{b^2-2av_0}/a
	//1/6at^3 + 1/2bt^2 + v_0t + x_0-x_1 = 0
	Vec3 a ={0,0,0};
	Vec3 b ={0,0,0};
	double t = Vec3_dist(pos, end)/(acc*acc);
	int hit = 0;
retry:
	for(int i =0; i<512; i++){
		const double x_x = 1./6. * a.x*t*t*t + 0.5 *b.x*t*t + vel.x* t + pos.x-end.x;
		const double x_y = 1./6. * a.y*t*t*t + 0.5 *b.y*t*t + vel.y* t + pos.y-end.y;
		const double x_z = 1./6. * a.z*t*t*t + 0.5 *b.z*t*t + vel.z* t + pos.z-end.z;	
		const double dx_xda = 1./6.*t*t*t;
		const double dx_yda = 1./6.*t*t*t;
		const double dx_zda = 1./6.*t*t*t;
		a.x -= x_x /(dx_xda)*0.1;
		a.y -= x_y /(dx_yda)*0.1;
		a.z -= x_z /(dx_zda)*0.1;
		const double dx_xdb = 0.5*t*t;
		const double dx_ydb = 0.5*t*t;
		const double dx_zdb = 0.5*t*t;
		b.x -= x_x/(dx_xdb)*0.1;
		b.y -= x_y/(dx_ydb)*0.1;
		b.z -= x_z/(dx_zdb)*0.1;	
	}
	for(int i =0; i<1024; i++){
		const double v_x =  0.5 * a.x* t*t +b.x*t + vel.x-des_vel.x; 
		const double v_y =  0.5 * a.y* t*t +b.y*t + vel.y-des_vel.y;
		const double v_z =  0.5 * a.z* t*t +b.z*t + vel.z-des_vel.z;
		const double dv_xdb = t;
		const double dv_ydb = t;
		const double dv_zdb = t;
		const double da_xdb = 1./3. *t;
		const double da_ydb = 1./3. *t;
		const double da_zdb = 1./3. *t;
		const double db_x = v_x/(dv_xdb)*0.1;
		const double db_y = v_y/(dv_ydb)*0.1;
		const double db_z = v_z/(dv_zdb)*0.1;
		b.x+= db_x;
		b.y+= db_y;
		b.z += db_z;
		a.x -= db_x /da_xdb;
		a.y -= db_y/da_ydb;
		a.z-= db_z/da_zdb;	
	}
	double max_acc = Vec3_len(b);
	double ac2 = Vec3_len(Vec3_add(Vec3_scale(a, t),b));
	if(ac2>max_acc){
		max_acc = ac2;
	}
	if(max_acc>acc){
		t*= 2;
		goto retry;
	}	
	if(max_acc<acc/1.1&& hit<5){
		hit += 1;
		t /= 2.0;
		goto retry;
	}
/*	if(Vec3_dist(posf, pos)>5.0){
		printf("unreachable position,velocity combination with linear acceleration\n");
//		assert(0);
	}*/

	return b;
}
Vec3 random_vec(int amount){
	return (Vec3){rand()%(amount*2)-amount, rand()%(amount*2)-amount, rand()%(amount*2)-amount};
}
void simulate(){
	Vec3 pos = random_vec(10000);
	Vec3 start = pos;
	Vec3 end = random_vec(10000);
	Vec3 vel = random_vec(10);
//	vel= (Vec3){0,0,0};
//	pos= (Vec3){0, 0,0};
//	end = (Vec3){40,0, 0};
	double acc = 50;
	double dt = 1./60.0;
	size_t count = 0;
	while(1){
		if(Vec3_dist(pos, end)<0.1){
			break;
		}
		Vec3 imp = next_impulse(pos, end, vel, (Vec3){0,0,0},acc);
		vel = Vec3_add(vel, Vec3_scale(imp,dt));
		pos = Vec3_add(pos, Vec3_scale(vel, dt));		
		if(count%60== 0){
			printf("pos:%f %f %f, end: %f %f %f, vel: %f %f %f\n", pos.x, pos.y, pos.z, end.x, end.y,end.z, vel.x, vel.y, vel.z); 
		}	
		count+=1;
	}
	printf("pos:%f %f %f, end: %f %f %f, vel: %f %f %f\n", pos.x, pos.y, pos.z, end.x, end.y,end.z, vel.x, vel.y, vel.z);
	printf("travelled %f meters in %f seconds, averaging:%f meters per second\n", Vec3_dist(start, end), (double)count/60.0, Vec3_dist(start, end)/((double)count/60.0));
	printf("count:%zu\n", count);
}
int main(void){	
	srand(time(0));
	//next_impulse((Vec3){0,0,0}, (Vec3){10,0,0}, (Vec3){0,0,0}, 10);
	simulate();
}
