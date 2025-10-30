#pragma once
/*
 * Use #define LOLTH_IMPLEMENTATION IN ONE .C FILE 
 * */
#include <stdio.h>
#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
	 void* ptr;
	 void *(*alloc)(void *, size_t count);
	 void (*dealloc)(void*, void*);
}Allocator;

typedef struct{	
	char * allocation;
	size_t head;
	size_t end;
	size_t capacity;
}Stream;
typedef enum{
	SEEK_FROM_START, 
	SEEK_FROM_END, 
	SEEK_FROM_CURRENT,
}SeekMode;
Stream stream_create(void);
size_t stream_write(Stream * stream, void * ptr, size_t count);
size_t stream_read(Stream * stream, void * ptr, size_t count);
size_t stream_seek(Stream * stream, int offset, SeekMode mode);
size_t stream_size(Stream * stream);
size_t stream_remaining(Stream * stream);	
size_t stream_write_u64(Stream * stream, uint64_t u);
uint64_t stream_read_u64(Stream * stream);
void stream_destroy(Stream * stream);
void stream_write_to_file(Stream * stream, FILE * file);
Stream stream_from_file(const char * path);
void stream_write_to_file_as_hex(Stream * stream, FILE * file);
extern Allocator std_allocator;
void * allocator_alloc(Allocator al, size_t size);
void allocator_dealloc(Allocator al,void * ptr);
typedef enum {
	TypeInt, 
	TypeUInt,
	TypeFloat,
	TypeString, 	
	TypeSlice,
	TypeStruct,
	TypeFunction
}TypeInfo;
struct Type;
typedef struct{
	const struct Type * info;
	size_t offset;
	const char * length_field_name;
	size_t indirection_count;
	size_t count;
	const char * name;
}Field;
typedef struct Type{
	TypeInfo info;
	size_t size;
	size_t align;
	size_t field_count;
	const Field * composition;	
	const char * name;
} Type;


#define REFLECT_VALUE __attribute__((unused)) static
#define INT_TYPE(SZ,NAME) REFLECT_VALUE const Type NAME##INFO=  (const Type){.info = TypeInt,.size = SZ, .align = SZ, .composition = 0, .name = #NAME};
#define FLOAT_TYPE(SZ,NAME) REFLECT_VALUE const Type NAME##INFO=  (const Type){.info = TypeFloat,.size = SZ, .align = SZ, .composition = 0,  .name = #NAME};
#define UINT_TYPE(SZ,NAME) REFLECT_VALUE const Type UNSIGNED_##NAME##INFO=  (const Type){.info = TypeUInt,.size = SZ, .align = SZ, .composition = 0, .name = "unsigned_"#NAME};

INT_TYPE(1, char)
INT_TYPE(sizeof(bool), bool)
INT_TYPE(2, short)
INT_TYPE(4, int)
INT_TYPE(8, long)
UINT_TYPE(1, char)
UINT_TYPE(2, short)
UINT_TYPE(4, int)
UINT_TYPE(8, long)
REFLECT_VALUE const Type size_tINFO=  (const Type){.info = TypeUInt,.size = sizeof(size_t), .align = sizeof(size_t), .composition = 0, .name = "size_t"};
FLOAT_TYPE(4, float)
FLOAT_TYPE(8, double)
REFLECT_VALUE const Type cstrINFO = (const Type){.info = TypeString, .size = sizeof(const char *), .align = sizeof(const char *), .composition = 0, .name = "cstr"};
#define sizeof_field(BASE_TYPE, NAME) sizeof(((BASE_TYPE*)0)->NAME)

#define REFLECT_FIELD(BASE_TYPE,TYPE,NAME) (const Field){.name = #NAME,.info= &TYPE##INFO, .offset = offsetof(BASE_TYPE, NAME),.indirection_count =0, .count = sizeof_field(BASE_TYPE, NAME)/sizeof(TYPE[1])}

#define REFLECT_PTR(BASE_TYPE,TYPE,NAME,COUNT) (const Field){.name = #NAME,.info = &TYPE##INFO, .offset = offsetof(BASE_TYPE, NAME),.indirection_count =COUNT, .count = 1}

#define REFLECT_CSTR(BASE_TYPE, NAME) (const Field){.name = #NAME, .info= &cstr##INFO, .offset = offsetof(BASE_TYPE, NAME), .indirection_count =0, .count =1}

#define REFLECT_SLICE(BASE_TYPE, NAME, TYPE) (const Field){.name = #NAME, .info = &TYPE##SliceINFO,.type = TypeSlice, .offset = offsetof(BASE_TYPE, NAME), .indirection_count = 0, .count =1)

#define REFLECT_FUNC_PTR(BASE_TYPE,NAME) (const Field){.name = #NAME, .info = 0, .type = TypeFunction, .offset = offsetof(BASE_TYPE, NAME), .count = 1}

#define REFLECT_STRUCT(NAME, ...) REFLECT_VALUE Type NAME##INFO = (const Type){.info = TypeStruct, .size = sizeof(NAME), .field_count = (sizeof((const Field[]){__VA_ARGS__})/sizeof(Field)),.composition = (const Field[]){__VA_ARGS__},.name = #NAME};


#define MAKE_SLICE(T) typedef struct{T* items; size_t count;} T##Slice; REFLECT_VALUE Type T##SliceInfo = (Type){.info = TypeSlice, .size = sizeof(T), .field_count = 0, .composition = 0};
typedef struct {
	const Type * info;
	size_t indirection_count;
	size_t count;
	void * ptr;
}Reflection;
#define REFLECT(T, V) (Reflection){.info =&T##INFO, .indirection_count = 0, .ptr = (void*)(V), .count = 1}

int reflect_get_int(Reflection a, int64_t * out);
int reflect_get_uint(Reflection a, uint64_t * out);
int reflect_get_float(Reflection a, double* out);
int reflect_get_string(Reflection a, const char ** out);
int reflect_get_field(Reflection a,int idx,Reflection * out);
int reflect_set_int(Reflection a, int64_t out);
int reflect_set_uint(Reflection a, uint64_t out);
int reflect_set_float(Reflection a, double out);
int reflect_set_cstr(Reflection a,const char * ptr);
int reflect_set_field(Reflection a, Reflection b, int idx);
int reflect_deref(Reflection a, Reflection * out);
void reflect_serialize(Stream *s, Reflection r);
void reflect_deserialize(Allocator al,Stream *stream, Reflection r);
void reflect_serialize_to_file(Reflection r, const char * path);
void reflect_deserialize_from_file(Allocator al,Reflection r,const char * path);
Reflection reflect_array_access(Reflection ref, int offset);
void debug_fprint(FILE * file,Reflection ref);
uint64_t reflect_array_size(Reflection ref);
int reflect_is_array(Reflection ref);
#ifdef LOLTH_IMPLEMENTATION
#include <stdlib.h>
#include <string.h>
Stream stream_create(void){
	Stream out;
	out.allocation = 0;
	out.capacity = 0;
	out.end = 0;
	out.head = 0;
	return out;
}

size_t stream_write(Stream * stream, void * ptr, size_t count){
	if(stream->capacity == 0){
		stream->allocation = malloc(1024);
		stream->capacity = 1024;		
		stream->head = 0;
		stream->end =0;
	}
	if(stream->head+count>= stream->capacity){
	grow_buffer:
		stream->capacity *= 2;	
		if(stream->head+count>= stream->capacity){
			goto grow_buffer;
		}
		stream->allocation = realloc(stream->allocation, stream->capacity);
		if(stream->allocation == 0){
			return 0;
		}
	}
	char * p = ptr;
	if(!p){
		return 0;
	}
	if(stream->head+count>= stream->end){
		stream->end = stream->head+count;
	}
	memcpy(stream->allocation+stream->head, ptr, count);
	stream->head += count;
	return count;
}
size_t stream_read(Stream * stream, void * ptr, size_t count){
	size_t to_read = 0;
	if(stream->head+count>= stream->end){
		to_read = stream->end-stream->head;
	}else{
		to_read = count;
	}
	memcpy(ptr, stream->allocation+stream->head,to_read);
	stream->head+= to_read;
	return to_read;
}

size_t stream_seek(Stream * stream, int offset, SeekMode mode){
	int64_t ptr = stream->head;
	if(mode == SEEK_FROM_CURRENT){
		ptr = stream->head+offset;
	}else if(mode == SEEK_FROM_END){
		ptr = stream->end+offset;
	}else if(mode == SEEK_FROM_START){
		ptr = offset;
	}
	if(ptr>= 0 && ptr< (int64_t)(stream->end)){
		stream->head = ptr;
	}
	return stream->head;
}
size_t stream_size(Stream * stream){
	return stream->end;
}
size_t stream_remaining(Stream * stream){
	if(stream->head>stream->end){
		return 0;
	}
	return stream->end-stream->head;
}
void stream_destroy(Stream * stream){
	if(stream->allocation){
		free(stream->allocation);
	}
	stream->capacity = 0;
	stream->end = 0;
	stream->head = 0;
	stream->allocation= 0;
}
void stream_write_to_file(Stream * stream, FILE * file){
	fwrite(stream->allocation, 1, stream->end, file);
}
void stream_write_to_file_as_hex(Stream * stream, FILE * file){
	for(size_t i =0; i<stream->end; i++){
		fprintf(file, "%x ",(int)stream->allocation[i]);	
	}
}
size_t stream_write_u64(Stream * stream, uint64_t u){
	return stream_write(stream, &u, sizeof(u));
}

uint64_t stream_read_u64(Stream * stream){
	uint64_t u =  0;
	stream_read(stream, &u, sizeof(u));
	return u;
}
Stream stream_from_file(const char * path){
	Stream out;
	FILE * f = fopen(path, "rb");
	fseek(f, 0, SEEK_END);
	int l = ftell(f);
	char * ptr = malloc(l);
	fseek(f, 0,SEEK_SET);
	fread(ptr, 1, l, f);
	fclose(f);
	out.allocation = ptr;
	out.capacity = l;
	out.end = l;
	out.head = 0;
	return out;
}
static void * wrap_malloc(void * in, size_t count){
	(void) in;
	return malloc(count);
}
static void wrap_free(void * in, void * ptr){
	(void) in;
	free(ptr);
}
 Allocator std_allocator = {.ptr = 0, .alloc = wrap_malloc, .dealloc = wrap_free};
void * allocator_alloc(Allocator al, size_t size){
	return al.alloc(al.ptr, size);
}
void allocator_dealloc(Allocator al,void * ptr){
	al.dealloc(al.ptr, ptr);
}

int reflect_get_int(Reflection a, int64_t * out){
	if(a.info->info != TypeInt){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 1){
		*out = *(const char*)(a.ptr);
	}else if(a.info->size == 2){
		*out = *(const short*)(a.ptr);
	}else if(a.info->size == 4){
		*out = *(const int*)(a.ptr);
	}else if(a.info->size == 8){
		*out = *(const int64_t*)(a.ptr);
	}else{
		return 1;
	}
	return 0;
}
int reflect_get_uint(Reflection a, uint64_t * out){
	if(a.info->info != TypeUInt){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 1){
		*out = *(const unsigned char*)(a.ptr);
	}else if(a.info->size == 2){
		*out = *(const unsigned short*)(a.ptr);
	}else if(a.info->size == 4){
		*out = *(const unsigned int*)(a.ptr);
	}else if(a.info->size == 8){
		*out = *(const uint64_t*)(a.ptr);
	}else{
		return 1;
	}
	return 0;
}
int reflect_get_float(Reflection a, double* out){
	if(a.info->info != TypeFloat){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 4){
		*out = *(const float*)(a.ptr);
	}else if(a.info->size == 8){
		*out = *(const double*)(a.ptr);
	}else{
		return 1;
	}
	return 0;
}
int reflect_get_string(Reflection a, const char ** out){
	if(a.info->info != TypeString){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	*out = *(const char**)a.ptr;
	return 0;
}
int reflect_get_field(Reflection a,int idx,Reflection * out){
	if(a.info->info != TypeStruct){
		return 1;
	}
	if((size_t)idx>= a.info->field_count){
		return 1;
	}
	if(idx<0){
		return 1;
	}
	char * p = (char*)a.ptr;
	Field info = a.info->composition[idx];
	out->info = info.info;
	out->indirection_count = info.indirection_count;
	out->ptr = p+info.offset;
	out->count = info.count;
	return 0;
}

int reflect_set_int(Reflection a, int64_t out){
	if(a.info->info != TypeInt){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 1){
		 *( char*)(a.ptr) = out;
	}else if(a.info->size == 2){
		 *( short*)(a.ptr) = out;
	}else if(a.info->size == 4){
		*( int*)(a.ptr) = out;
	}else if(a.info->size == 8){
		*( int64_t*)(a.ptr) = out;

	}else{
		return 1;
	}
	return 0;
}
int reflect_set_uint(Reflection a, uint64_t out){
	if(a.info->info != TypeUInt){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 1){
		 *( unsigned char*)(a.ptr) = out;
	}else if(a.info->size == 2){
		 *( unsigned short*)(a.ptr) = out;
	}else if(a.info->size == 4){
		*( unsigned int*)(a.ptr) = out;
	}else if(a.info->size == 8){
		*( uint64_t*)(a.ptr) = out;

	}else{
		return 1;
	}
	return 0;
}
int reflect_set_float(Reflection a, double out){
	if(a.info->info != TypeFloat){
		return 1;	
	}
	if(a.indirection_count!= 0){
		return 1;
	}
	if(a.info->size == 4){
		*( float*)(a.ptr) = out;
	}else if(a.info->size == 8){
		*( double*)(a.ptr) = out;

	}else{
		return 1;
	}
	return 0;
}
int reflect_set_cstr(Reflection a,const char * ptr){
	if(a.info->info != TypeString){
		return 1;
	}
	*(const char**)a.ptr = ptr;
	return 0;
}
int reflect_set_field(Reflection a, Reflection b, int idx){
	if(a.info->info != TypeStruct){
		return 1;
	}
	if(a.info->field_count>=(size_t)idx){
		return 1;
	}
	if(idx<0){
		return 1;
	}
	char * p = (char*)a.ptr;
	Field info = a.info->composition[idx];
	char * f = p+info.offset;
	if(info.indirection_count != 0){
		memcpy(f, b.ptr, sizeof(void*));
	}else{
		memcpy(f, b.ptr, info.info->size * info.count);
	}
	return 0;
}
int reflect_deref(Reflection a, Reflection * out){
	if(a.indirection_count==0){
		return 1;
	}
	Reflection b = a;
	b.indirection_count -=1;
	b.ptr = *(void**)b.ptr;
	*out = b;
	return 0;
}
void debug_fprint(FILE * file,Reflection ref){
	 Reflection rf = ref;
	 if(rf.indirection_count){	
	 	do{
		 	if(*(void**)rf.ptr == 0){
				fprintf(file, "NULL");
				return; 
	 		}
	 		if(rf.indirection_count>0){
				reflect_deref(rf, &rf);
	 		}
	 	}while(rf.indirection_count>1);	
	 }
	if(reflect_is_array(rf)){
		long l = reflect_array_size(rf);
		fprintf(file, "[");
		for(int i =0; i<l ; i++){
			Reflection r = reflect_array_access(rf, i);
			debug_fprint(file, r);
			if(i != l-1){
				fprintf(file, ",");
			}
		}
		fprintf(file, "]");

	} else if(rf.info->info== TypeInt){
		int64_t s = 0;
	 	reflect_get_int(rf, &s);
		if(rf.info->size == 1){
			fprintf(file, "%c", (char)s);
		}else{
	 		fprintf(file, "%ld", (long)s);
		}

	 }else if(rf.info->info == TypeUInt){
		uint64_t s = 0;
	 	reflect_get_uint(rf, &s);
	 	fprintf(file, "%lu", (unsigned long)s);

	 }else if(rf.info->info == TypeFloat){
	 	double s = 0.0;
	 	reflect_get_float(rf, &s);
	 	fprintf(file, "%f", s);
	 }else if(rf.info->info == TypeString){
	 	const char * s = 0;
	 	reflect_get_string(rf, &s);
	 	fprintf(file,"\"%s\"", s);
	 }else if(rf.info->info== TypeSlice){
	 	return;
	 }else if(rf.info->info == TypeStruct){
		fprintf(file,"%s:{", rf.info->name);
	 	for(size_t i =0; i<rf.info->field_count; i++){
	 		fprintf(file,"%s = ", rf.info->composition[i].name);
	 		Reflection r;
	 		reflect_get_field(rf, i, &r);
	 		debug_fprint(file, r);
	 		if(i != rf.info->field_count-1){
	 			fprintf(file, ",");
	 		}
		
	 	}
	 	fprintf(file, "}");
	 }else{
	 	exit(-1);
	 	return;
	 }
}
void reflect_serialize(Stream *stream, Reflection r){
	 Reflection rf = r;
	 if(rf.indirection_count){	
	 	do{
		 	if(*(void**)rf.ptr == 0){	
				stream_write_u64(stream, 0);
				return; 
	 		}
	 		if(rf.indirection_count>0){
				reflect_deref(rf, &rf);
	 		}
	 	}while(rf.indirection_count>1);	
		stream_write_u64(stream, 1);
	 }
	if(reflect_is_array(r)){
		long l= reflect_array_size(r);
		stream_write_u64(stream,l);
		for(long i =0; i<l ;i++){
			reflect_serialize(stream, reflect_array_access(r, i));
		}
	}
	else if(rf.info->info== TypeInt){
		int64_t s = 0;
		reflect_get_int(rf, &s);	 		
		stream_write(stream, &s, sizeof(s));	
	 }else if(rf.info->info == TypeUInt){	
		uint64_t s = 0;
		reflect_get_uint(rf, &s);
		stream_write(stream, &s, sizeof(s));	
	 }else if(rf.info->info == TypeFloat){
	 	double s = 0.0;
		reflect_get_float(rf, &s);
		stream_write(stream, &s, sizeof(s));	
	 }else if(rf.info->info == TypeString){
	 	const char * s = 0;
	 	reflect_get_string(rf, &s);
 		uint64_t l = 0;
		if(s != 0){
			l = strlen(s);
		}
		stream_write(stream, &l, sizeof(l));
		if(l != 0){
			stream_write(stream, (void*)s, l);
		}		
	 }else if(rf.info->info == TypeStruct){
		stream_write_u64(stream,strlen(rf.info->name));
		stream_write(stream, (void*)rf.info->name, strlen(rf.info->name));
	 	for(size_t i =0; i<rf.info->field_count; i++){	
	 		Reflection r;
	 		reflect_get_field(rf, i, &r);	
			reflect_serialize(stream, r);
	 	}	
	 }else{
	 	exit(-1);
	 	return;
	 }

}

void reflect_deserialize(Allocator al,Stream *stream, Reflection r){
	 Reflection rf = r;
	 if(rf.indirection_count){	
		uint64_t t = stream_read_u64(stream);
		void * ptr = 0;
		if(t){
			Reflection rt= rf;
			rt.indirection_count -=1;
			rt.ptr = allocator_alloc(al,rt.info->size);
			reflect_deserialize(al,stream, rt);
			*(void**)rf.ptr = rt.ptr;
			for(size_t i =0; i<rf.indirection_count-1; i++){
				
				void ** ptr = allocator_alloc(al,sizeof(void*));
				*ptr = *(void**)rt.ptr;
				*(void**)rt.ptr = ptr;
				rt.indirection_count-=1;
			}
			return;
		}else{
			*(void**)rf.ptr =0;
			return;
		}
	 }
	if(reflect_is_array(r)){
		long l = stream_read_u64(stream);
		if(r.info->info == TypeSlice){
			Reflection ptr,sz;
			reflect_get_field(r, 0, &ptr);
			reflect_get_field(r, 1, &sz);
			reflect_set_uint(sz, l);
			*(void**)ptr.ptr = allocator_alloc(al,ptr.info->size*l);
		}
		for(int i =0; i<l; i++){
			reflect_deserialize(al,stream,reflect_array_access(r, i));
		}
	
	}
	else if(rf.info->info== TypeInt){
			int64_t s = 0;
			stream_read(stream, &s, sizeof(s));
			reflect_set_int(rf, s);
	 }else if(rf.info->info == TypeUInt){
		uint64_t s = 0;
		stream_read(stream, &s, sizeof(s));
		reflect_set_uint(rf, s);	
	 }else if(rf.info->info == TypeFloat){
					double s = 0;
			stream_read(stream, &s, sizeof(s));
			reflect_set_float(rf, s);	
		
	 }else if(rf.info->info == TypeString){
	 			uint64_t l =stream_read_u64(stream);	
			char * ptr = 0;
			if(l != 0){
				ptr =allocator_alloc(al,l+1)	;
				ptr[l] =0;
				stream_read(stream, ptr, l);
			}
			reflect_set_cstr(rf, ptr);	
		}	
	 else if(rf.info->info == TypeStruct){
		int l = strlen(rf.info->name);
		long p = stream_read_u64(stream);
		if(l != p){
			return;
		}
		char * name = allocator_alloc(al,l);
		stream_read(stream, name, l);
		for(int i =0; i<l ; i++){
			if(name[i] != rf.info->name[i]){
				return;
			}
		}
		allocator_dealloc(al, name);	
	 	for(size_t i =0; i<rf.info->field_count; i++){	
	 		Reflection r;
	 		reflect_get_field(rf, i, &r);	
			reflect_deserialize(al,stream, r);
	 	}	
	 }else{
	 	exit(-1);
	 	return;
	 }

}

int reflect_is_array(Reflection ref){
	return ref.count>1;
}
Reflection reflect_array_access(Reflection ref, int offset){
	Reflection out = ref;
	if(ref.info->info == TypeSlice){	
		Reflection ptr;
		const Type * t = ref.info->composition[0].info;
		reflect_get_field(ref, 0, &ptr);
		char * p = *(void**)ptr.ptr;
		char * out =p+ t->size*offset;
		Reflection o;
		o.indirection_count = ref.indirection_count;
		o.count = 1;
		o.info = t;
		o.ptr = out;
		return o;
	}else{	
		const Type * t= ref.info;
		char * p = ref.ptr;
		char * out = p+t->size*offset;
		Reflection o;
		o.count = 1;
		o.indirection_count = ref.indirection_count;
		o.info = t;
		o.ptr = out;
		return o;
	}
}
uint64_t reflect_array_size(Reflection ref){
	if(ref.info->info == TypeSlice){
		Reflection sz;
		reflect_get_field(ref, 1, &sz);
		uint64_t p;
		reflect_get_uint(sz, &p);
		return p;
	}else{
		return ref.count;
	}
}
void reflect_serialize_to_file(Reflection r, const char * path){
	Stream s = stream_create();
	reflect_serialize(&s, r);
	FILE * f = fopen(path, "wb");
	stream_write_to_file(&s, f);
	fclose(f);
	stream_destroy(&s);
}
void reflect_deserialize_from_file(Allocator al,Reflection r,const char * path){
	Stream s = stream_from_file(path);
	reflect_deserialize(al,&s, r);
	stream_destroy(&s);
}

#endif
