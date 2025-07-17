#include "cereal.h"
#include "interface.h"
class Person{
	std::string name;
	size_t age;	
	std::vector<std::string> inventory;
public:
	Person(){
		name = "jimbob";
		age = 42069;	
		static const std::vector<std::string> invent= {"piano", "ak-47", "yaoi"};
		inventory = invent;
	};
	void print(){
		printf("name:%s\nage:%zu\ninventory:\n",name.c_str(), age);
		size_t inventory_count = inventory.size();
		for(size_t i =0; i<inventory_count; i++){
			printf("%s", inventory[i].c_str());
			if(i != inventory_count-1){
				printf(",");
			}	
		}
		printf("\n");
	}
	void serialize(Serializer* ser)const{
		ser->serialize(name);
		ser->serialize(age);
		ser->serialize_array(inventory.data(), inventory.size());
	}
	static Person deserialize(Deserializer * des){
		Person out;
		out.name = des->deserialize<std::string>();
		out.age = des->deserialize<size_t>();
		out.inventory = des->deserialize_array<std::string>();
		return out;
	}
	
};
class Printer:public Interface{
	public: 
	~Printer(){}
	std::string name;
	virtual void print(){
		printf("%s\n", name.c_str());
	}
	static Printer deserialize(Deserializer* des) {
		Printer out;
		des->deserialize<std::string>();
		out.name = des->deserialize<std::string>();
		return out;
	}
	static Interface * interface_deserialize(Deserializer& des){
		return new Printer(deserialize(&des));
	}
	virtual void serialize(Serializer* ser)const {
		ser->serialize("Printer");
		ser->serialize(name);
	}
};
Register(Printer, Interface);
extern Interface* generate(long long v);
void writer(){
	Printer * print = new Printer;
	print->name = "hello world";
	Interface * p = print;
	Serializer s;
	s.serialize_interface(p);
	s.serialize_interface(generate(32));
	s.write_to_file("test.bin");
}

void reader(){
	Deserializer ds = Deserializer::from_file("test.bin");
	Interface * f =ds.deserialize_interface<Interface>();
	Interface * k = ds.deserialize_interface<Interface>();
	f->print();
	k->print();	
}
void test(){
	std::vector<uint32_t> vec;
	for(uint32_t i =0; i<1000; i++){
		vec.push_back(i);
	}
	Serializer s;
	s.serialize_array(vec.data(), vec.size());
	auto a = s.as_bytes();
	Deserializer d= Deserializer(a.data(), a.size());
	std::vector<uint32_t> vs =d.deserialize_array<uint32_t>();
	for(size_t i =0; i<vs.size(); i++){
		printf("%u\n", vs[i]);
	}
	s.write_to_file("test.bin");
}
void test2(){
	Deserializer d= Deserializer::from_file("test.bin");
	std::vector<uint32_t> vs = d.deserialize_array<uint32_t>();
	std::vector<uint32_t> ts;
	for(size_t i =0; i<vs.size(); i++){
		printf("%u\n", vs[i]);
		ts.push_back(ts[i]);
	}
	Serializer s;
	s.serialize_array<uint32_t>(ts.data(), ts.size());
	s.write_to_file("test.bin");
}
int main(void){		
	writer();
	reader();	
	exit(0);
	//test2();
}
