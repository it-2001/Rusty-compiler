// here i test behavior of parser for variables and expressions

const ABOMINATION: int = 1
                        + -50
                        * danda.getName[5+6]<sedm>(NameTypes.FIRST) as int 
                        / (5f) as int
const CIRNO: string = "fumo"
const BOOL: bool = true
const ARRAY: [int; 3] = [1, 2, 3]
const ARRAY_BUILDER: [int; 3] = [5; 15]
const DYNAMIC_ARRAY: &[int; _] = new [1, 2, 3]
const DYNAMIC_ARRAY_BUILDER: &[int; _] = new [5; 15]
// test for generics
const GENERIC_TYPE: Something<int> = Something(1)
// test for generics with traits
const GENERIC_TYPE_WITH_TRAITS: Something<int> = Something(1)
// structs
struct Something<T> {
    value: T
}
// implementation of struct Something
impl Something {
    fun constructor(value: T): Self<T> {
        self.value = value
    }
}
// traits
trait Trait {
    fun method(): int
    overload + (other: Self): Self
}
// functions assigned to constants
const FUNCTION: fun(): int = fun(): int {
    try { } catch e: Exception {
        return 1
    } finally {
        return 2
    }
}
// function
const FUNCTION_ARRAY: [fun(): int; 3] = [fun(): int {
    return 7
}, fun(): int {
    return 8
}, fun(): int {
    return 9
}]
// 2d array
const ARRAY_2D: [[int; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]

// error declaration
error BadId(id: int, maxId: int) {
    message: "Expected id between 0 and " + maxId + ", got " + id,
    code: 1
}