/*!
# || -> Result<_,_> {}
Oh. My. Gosh.
I did not realize that we could specify the return types of closures!
This makes working with Results and iterators *so* much simpler!
*/
fn main() {
        #[expect(clippy::single_range_in_vec_init)]
        let v = [0..10];
        // let x = v[0];
        let [x] = v;
        #[expect(clippy::manual_inspect)]
        let y = x.map(|x| {
                println!("{:?}", x);
                x
        });
        println!("-------After 'map()', before 'sum()'---------");
        let z: i32 = y.sum();
        dbg!(z);
        println!("- - - - - - - -");
        let resmap = (0..10).map(|x| -> Result<i32, Box<dyn std::error::Error>> {
                if x % 2 == 0 { Ok(x) } else { Err("Odd number".to_string())? }
        });
        let resvec = resmap.collect::<Vec<_>>();
        dbg!(resvec);

        println!("- - - - - - - -");
        let resmap = (0..10).map(|x| -> Result<i32, Box<dyn std::error::Error>> {
                if x % 2 == 0 { Ok(x) } else { Err("Odd number".to_string())? }
        });
        let resmapfilter = resmap.filter(|r| r.is_ok());
        let resfiltervec = resmapfilter.collect::<Vec<_>>();
        dbg!(resfiltervec);

        println!("- - - - - - - -");
        let resmap = (1..=10).map(|x| -> Result<i32, Box<dyn std::error::Error>> {
                if x % 3 != 0 { Ok(x) } else { Err("Odd number".to_string())? }
        });
        let resmapmapwhile = resmap.map_while(|r| r.ok());
        let reswhilevec = resmapmapwhile.collect::<Vec<_>>();
        dbg!(reswhilevec);
}
