#pragma once
#include "cereal.h"
class Interface{
	public:
	virtual void print() =0;
	inline virtual ~Interface(){}
	virtual void serialize(Serializer* ser)const = 0;
};

