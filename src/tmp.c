typedef void (*Udf_func_clear)(UDF_INIT *, unsigned char *, unsigned char *);
typedef void (*Udf_func_add)(UDF_INIT *, UDF_ARGS *, unsigned char *,
                             unsigned char *);

typedef void (*Udf_func_deinit)(UDF_INIT *);
typedef bool (*Udf_func_init)(UDF_INIT *, UDF_ARGS *, char *);
typedef void (*Udf_func_any)(void);

typedef double (*Udf_func_double)(UDF_INIT *, UDF_ARGS *, unsigned char *,
                                  unsigned char *);
typedef long long (*Udf_func_longlong)(UDF_INIT *, UDF_ARGS *, unsigned char *,
                                       unsigned char *);
typedef char *(*Udf_func_string)(UDF_INIT *, UDF_ARGS *, char *,
                                 unsigned long *, unsigned char *,
                                 unsigned char *);
