fn add(c:i32, d:i32) -> i32 {
	       return c+d;
        }

fn main() {
	       let a:i32 = 1;
	       let b:i32 = 2;

	       let c:i32 = add(a, b);
           if b > c {
              c = 10;
           };
        }