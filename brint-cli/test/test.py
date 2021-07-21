@brint.feature(name='my feature1', version='1.3.4')
def function_1():
    return 3


@brint.feature(name='my feature2', version='1.0.0')
def function_2():
    return 3


@brint.feature(name='my feature3', version='1.3.6', new=sdf)
def function_3():
    return 3


@brint.feature(name='my feature4', version='1.3.6', old=fnuction_1)
def function_34():
    return 3
