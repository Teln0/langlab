use langlab::lexical::automata::{FiniteAutomata};
use langlab::lexical::regex::RegEx;

fn main() {
    let input = "ab*c";
    println!("INPUT DUMP\n{}", input);

    let regex = RegEx::new_from_chars(&mut input.chars());
    println!("\n\nREGEX DUMP\n{:?}", regex);

    let nfa = FiniteAutomata::new_from_regex(&regex);
    println!("\n\nNFA DUMP");
    nfa.dump();

    let dfa = nfa.nfa_to_dfa();
    println!("\n\nDFA DUMP");
    dfa.dump();
}
