use langlab::lexical::automata::{FiniteAutomata};
use langlab::lexical::regex::RegEx;

fn main() {
    let input = "ab*c";
    let regex = RegEx::new_from_chars(&mut input.chars());
    let nfa = FiniteAutomata::new_from_regex(&regex);
    let dfa = nfa.nfa_to_dfa();
    println!("INPUT DUMP\n{}", input);
    println!("\n\nREGEX DUMP\n{:?}", regex);
    println!("\n\nNFA DUMP");
    nfa.dump();
    println!("\n\nDFA DUMP");
    dfa.dump();
}
