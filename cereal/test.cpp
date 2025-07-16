#include "cereal.h"
#include "interface.h"
class Number: public Interface{	
public:
	long long number;
	virtual ~Number(){}
	virtual void print(){
		printf("number is %lld\n", number);
	}
	virtual void serialize(Serializer* ser)const{
		ser->serialize("Number");
		ser->serialize(number);
	}
	static Number deserialize(Deserializer * des){
		Number out;
		des->deserialize<std::string>();
		out.number = des->deserialize<long long>();
		return out;	
	}
	static Interface * interface_deserialize(Deserializer&des){
		return new Number(deserialize(&des));
	}
};
Register(Number,Interface);
Interface* generate(long long v){	
	Number * out = new Number();	
	out->number = v;
	return out;
}
