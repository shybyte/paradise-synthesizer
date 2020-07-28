use soundpipe::Soundpipe;
use soundpipe::factory::Factory;

fn main() {
    println!("Hello, world!");
    let soundpipe = Soundpipe::new(44100);
    let saw = soundpipe.bl_saw();
    let out = saw.compute();
    eprintln!("value = {:?}", out);

}
