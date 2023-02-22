# JustLinked

fast linked list;give you a different feeling!

```rust
use just_linked::LURLinked;

fn main() {
    let mut lur = LURLinked::new();

    let one = lur.push(1);
    let tow = lur.push(2);
    let three = lur.push(3);

    assert_eq!(lur.get(one),Some(&1));
    assert_eq!(lur.get_mut(tow),Some(&mut 2));
    
    lur.move_front(one).unwrap();
    assert_eq!(lur.remove(one), Some(1));
    lur.move_front(tow).unwrap();
    assert_eq!(lur.remove(tow), Some(2));
    lur.move_front(three).unwrap();
    assert_eq!(lur.remove(three), Some(3));

    lur.push(1);
    lur.push(2);
    lur.push(3);

    assert_eq!(lur.remove_last(), Some(1));
    assert_eq!(lur.remove_last(), Some(2));
    assert_eq!(lur.remove_last(), Some(3));

    for i in 1..10i32 {
        lur.push(i);
    }

    for (_, item) in lur.iter_mut() {
        *item += 1;
    }

    for (key, value) in lur.iter() {
        println!("key:{},value:{}",key,value);
    }
    
    lur.clear();
}

```