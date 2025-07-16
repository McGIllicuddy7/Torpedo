#pragma once 
#include <cstdlib>
#include <type_traits>
#include <vector>
#include <string>
#include <functional>
#include <unordered_map>
#include <assert.h>
template <typename T> concept Trivial= std::is_trivial_v<T>;


struct SerialValue{
	std::string name;	
	size_t count;
	unsigned char * buffer;	
};
class Serializer{
	std::vector<SerialValue> values;
public:
	void push_value(SerialValue value);
	~Serializer();
	Serializer();
	template<typename T> void serialize_trivial(const T& v){
		static_assert(std::is_trivially_copyable_v<T>);
		unsigned char * chars = new unsigned char[sizeof(T)];
		memcpy(chars, &v, sizeof(T));
		SerialValue s;
		s.count = sizeof(T);
		s.buffer = chars;
		push_value(s);
	}	
	template <typename T> void serialize_trivial_array(const T* base, size_t count){
		static_assert(std::is_trivially_copyable_v<T>);
		unsigned char * chars = new unsigned char[sizeof(T)*count];
		memcpy(chars, base, sizeof(T)*count);
		SerialValue s;
		s.count = sizeof(T)*count;
		s.buffer = chars;
		s.name = "";
		push_value(s);	
	}	
	template <typename T> void serialize_trivial_ptr(const T* base){
		static_assert(std::is_trivially_copyable_v<T>);
		serialize((bool)base);	
		if(!base){
			return;
		}
		unsigned char * chars = new unsigned char[sizeof(T)];
		memcpy(chars, base, sizeof(T));
		SerialValue s;
		s.count = sizeof(T);
		s.buffer = chars;
		push_value(s);	
	}	
	template <typename T>void  serialize_interface(const T* object){
		serialize((bool)object);
		if(object){
			object->serialize(this);	
		}	
	}
	template <typename T> void serialize_array(const T* base, size_t count){
		serialize(count);
		for(size_t i =0; i<count; i++){
			serialize(base[i]);
		}
	}
	template<Trivial T> void serialize_array(const T* base, size_t count){
		serialize_trivial_array(base,count);
	}
	template <typename T> void serialize_interface_array(const T* base, size_t count){
		serialize(count);
		for(size_t i =0; i<count; i++){
			serialize_interface(base[i]);
		}
	}
	
	template<typename T> void serialize(const T& obj){
		call_serialize(this, obj);	
	}	
	template<Trivial T> static void vec_push_value(std::vector<unsigned char>& as, const T & value){
		const unsigned char * vptr = (const unsigned char *)&value;
		for(size_t i =0; i<sizeof(T); i++){
			as.push_back(vptr[i]);
		}
	}
	template<Trivial T> static void vec_push_array(std::vector<unsigned char>&as, const T * value, size_t count){
		size_t prev = as.size();
		vec_push_value(as, count);
		assert(as.size()== prev+8);
		for(size_t i = 0; i<count; i++){
			vec_push_value(as,value[i]);
		}
	}
	static void serialize_serial_value(std::vector<unsigned char> & as, SerialValue value);
	std::vector<unsigned char> as_bytes() const;
	void write_to_file(const char * file_name);
	template<typename T,typename U> void serialize_map(const std::unordered_map<T,U> v){
		serialize(v.size());
		for(auto& i:v){
			serialize(i.first);
			serialize(i.second);
		}
	}
	size_t get_current_idx() const;
};
template <Trivial T> void call_serialize(Serializer*__restrict__ ser, const T& v){
	ser->serialize_trivial(v);	
}
template <Trivial T> void call_serialize(Serializer*__restrict__ ser, const T* v, size_t count){
	ser->serialize_trivial_array(v, count);	
}
template <Trivial T> void call_serialize(Serializer*__restrict__ ser, const T* v){
	ser->serialize_trivial_ptr(v);	
}
template<typename T> void call_serialize(Serializer*__restrict__ ser, const T& v){
	v.serialize(ser);
}
template<typename T> void call_serialize(Serializer*__restrict__ ser, const T*v){
	v->serialize(ser);
}
void call_serialize(Serializer*__restrict__ ser,const char * ptr);
void call_serialize(Serializer* __restrict__ ser, const std::string& s);
template<typename T> class InterfaceTable;
class Deserializer{
	const unsigned char * buffer;
	size_t buffsz; size_t current_idx;
	const unsigned char * request_bytes(size_t count);
	SerialValue get_next_serial_value();
	SerialValue peek_next_serial_value();
public:
	Deserializer();
	Deserializer(Deserializer &) = delete;
	Deserializer(const Deserializer &) = delete;
	Deserializer& operator=(Deserializer&) = delete;
	Deserializer& operator=(const Deserializer&) = delete;
	Deserializer(const unsigned char * buffer, size_t buffsz);
	static Deserializer from_file(const char * path);
	size_t get_current_idx()const;
	template<typename T> T deserialize_trivial(){
		static_assert(std::is_trivially_copyable_v<T>);
		SerialValue v = get_next_serial_value();
		alignas(alignof(T)) char buffer[sizeof(T)];
		memcpy(buffer, v.buffer, sizeof(T));
		return *(T*)buffer;
		
	}	

	template <typename T> std::vector<T> deserialize_trivial_array(){	
		static_assert(std::is_trivially_copyable_v<T>);
		SerialValue val = get_next_serial_value();
		T* p = (T*)val.buffer;
		size_t count = val.count/sizeof(T);
		std::vector<T> out;
		for(size_t i =0; i<count; i++){
			out.push_back(p[i]);			
		}
		return out;
	}	
	template <typename T>T* deserialize_trivial_ptr(){
		static_assert(std::is_trivially_copyable_v<T>);
		bool valid = deserialize<bool>();
		if(!valid){
			return 0;
		}
		SerialValue v= get_next_serial_value();
		T* out = new T(*(T*)v.buffer);
		return out;	
	}	
	template<typename T> T* deserialize_interface(){	
		InterfaceTable<T> table = InterfaceTable<T>{};
		bool exists = deserialize<bool>();
		size_t tmp = current_idx;
		if(!exists){
			return 0;
		}
		std::string name = deserialize<std::string>();
		current_idx = tmp;
		return table.deserialize_value(*this, name);
	}

	template<typename T> T deserialize(){
		return T::deserialize(this);
	}	
	template<> std::string deserialize<std::string>(){
		SerialValue v= get_next_serial_value();
		std::string out;	
		for(size_t i =0; i<v.count; i++){
			out.push_back(v.buffer[i]);
		}	
		return out;
	}
	template<Trivial T> T deserialize(){
		return deserialize_trivial<T>();
	}
	template<Trivial T> T* deserialize_ptr(){
		return deserialize_trivial_ptr<T>();
	}
	template<Trivial T> std::vector<T> deserialize_array(){
		return deserialize_trivial_array<T>();
	}
	template<typename T> std::vector<T> deserialize_array(){
		size_t count = deserialize<size_t>();
		std::vector<T> out;
		for(size_t i =0; i<count; i++){	
			out.push_back(deserialize<T>());
		}
		return out;
	}
	template<typename T> std::vector<T*> deserialize_interface_array(){
		size_t count = deserialize<size_t>();
		std::vector<T*> out;
		for(size_t i =0; i<count; i++){
			out.push_back(deserialize_interface<T>());
		}
		return out;
	}	
	template<typename T, typename U> std::unordered_map<T,U> deserialize_hashmap(){
		size_t count = deserialize<size_t>();
		std::unordered_map<T, U> out;
		for(size_t i =0; i<count; i++){
			T a= deserialize<T>();
			U b = deserialize<U>();
			out.insert({a,b});
		}
		return out;
	}
};

template<typename T> class InterfaceTable{
	
public:
	static inline std::unordered_map<std::string,std::function<T*(Deserializer&)>> table = {};
	static void register_function(std::string name, std::function<T*(Deserializer&)>fn){
		table.insert({name, fn});
	}
	static void deregister_function(std::string name){
		table.erase(name);
	}
	static T* deserialize_value(Deserializer& des, std::string name){
		return (table[name])(des);
	}
};
template<typename T> size_t static_register_type(std::string name, std::function<T*(Deserializer&)>fn ){
	InterfaceTable<T>::register_function(name, fn);
	return InterfaceTable<T>::table.size();
}
#ifndef stringify
#define stringify(s) #s
#endif
#define Register(T,Int) const size_t T##register_index= static_register_type<Int>(stringify(T),T::interface_deserialize);

