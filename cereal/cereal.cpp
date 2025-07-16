#include "cereal.h"
void call_serialize(Serializer*__restrict__ ser,const char * ptr){
	size_t count = strlen(ptr);	
	ser->serialize_trivial_array(ptr, count);
}
void call_serialize(Serializer* __restrict__ ser, const std::string& s){
	SerialValue v;
	v.name = "std::string";
	v.count = s.length();
	v.buffer= new unsigned char[s.length()+1];
	memset(v.buffer,0, s.length()+1);
	memcpy(v.buffer, s.c_str(), s.length());
	ser->push_value(v);	
}
Serializer::~Serializer(){
		for(size_t i =0; i<values.size(); i++){
			delete values[i].buffer;
		}
	}
void Serializer::push_value(SerialValue value){
	values.push_back(value);

}
void Serializer::serialize_serial_value(std::vector<unsigned char> & as, SerialValue value){
		if(value.name.size() == 0){
			const char arr[] = {0};
			vec_push_array(as, arr, 1);
		}
		else{
			vec_push_array(as, value.name.c_str(), value.name.size());
		}	
		vec_push_array(as, value.buffer, value.count);
	}
std::vector<unsigned char> Serializer::as_bytes() const{
		std::vector<unsigned char> out ={};
		for(size_t i =0; i<values.size(); i++){
			serialize_serial_value(out, values[i]);	
		}
		return out;
}
Deserializer::Deserializer(const unsigned char * inbuffer, size_t inbuffsz){
	buffer = inbuffer;
	buffsz = inbuffsz;
	current_idx =0;
}
const unsigned char * Deserializer::request_bytes(size_t count){
	size_t new_count = current_idx+count;
	if(new_count>buffsz){
		printf("returned 0, buffsz:%zu, count:%zu\n",buffsz, new_count);
		throw new int(-1);
		return 0;
	}else{
		const unsigned char * out = &buffer[current_idx];
		current_idx = new_count;
		return out;
	}
}

SerialValue Deserializer::get_next_serial_value(){
	size_t count =*(size_t*)request_bytes(sizeof(size_t));
	const char * c = (const char*)request_bytes(count);
	std::string name;
	for(size_t i =0; i<count; i++){
		name += c[i];
	}	
	count = 0;
	const unsigned char * p = request_bytes(sizeof(size_t));			memcpy((unsigned char *)&count, p, sizeof(size_t));
	const unsigned char * buffer = request_bytes(count);	
	SerialValue v;
	v.buffer = (unsigned char *)buffer;
	v.name = std::move(name);
	v.count = count;
	return v;
}

SerialValue Deserializer::peek_next_serial_value(){
	size_t old_idx = current_idx;
	SerialValue out = get_next_serial_value();
	current_idx = old_idx;
	return out;
}
void Serializer::write_to_file(const char * file_name){
	FILE * f= fopen(file_name,"w");
	if(!f){
		perror("ERROR");
	}
	std::vector<unsigned char> bytes = as_bytes();
	fwrite(bytes.data(), bytes.size(), 1, f);
	fflush(f);
	fclose(f);
}
Deserializer Deserializer::from_file(const char * path){
	FILE * f = fopen(path, "r");
	if(!f){
		perror("ERROR");
		throw new std::string((std::string)"file " + path +"does not exist");
	}
	fseek(f,0, SEEK_END);
	size_t size = ftell(f);
	fseek(f, 0, SEEK_SET);
	unsigned char * buffer = new unsigned char[size];
	fread(buffer, size, 1,f);
	fclose(f);	
	return  Deserializer(buffer, size);
}
size_t Serializer::get_current_idx()const{
	return as_bytes().size();
}
size_t Deserializer::get_current_idx() const{
	return current_idx;
}
Serializer::Serializer(){
	values = {};
}
Deserializer::Deserializer(){
	buffer =0;
	buffsz =0;
	current_idx =0;

}