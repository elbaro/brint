import brint


def example1():
    def new_function():
        print('new function')

    @brint.feature(name='asdf', version='3.0.0', new=new_function)
    def function():
        print('old function')

    function()


def example2():
    class OldClass:
        def __init__(self):
            print('OldClass')

    @brint.feature(name='new class', version='4.0.0', old=OldClass)
    class Class:
        def __init__(self):
            print('NewClass')

    _ = Class()


example1()
example2()
