#pragma once

typedef signed char int8;
typedef signed short int16;
typedef signed int int32;
typedef signed long long int64;

typedef unsigned char uint8;
typedef unsigned short uint16;
typedef unsigned int uint32;
typedef unsigned long long uint64;

typedef signed long int isize;
typedef unsigned long int usize;

typedef _Bool bool;

#define true 1
#define false 0

typedef struct slice
{
    usize len;
    void *ptr;
} slice;

typedef struct str
{
    usize len;
    char *ptr;
} str;

#define array_access(lval, ty) _Generic((lval), slice: ((ty*)lval.ptr), default: lval)