use std::collections::{HashMap};
use crate::lexical::regex::RegEx;

pub struct StateTransition {
    // None represents an Epsilon transition
    pub condition: Option<char>,
    pub target_state: usize
}

pub struct StateTransitions {
    pub transitions: Vec<StateTransition>
}

/// Starting state is always at index 0
pub struct FiniteAutomata {
    pub accepting_states: Vec<usize>,
    pub state_transitions: Vec<StateTransitions>
}

// TODO use HashSets instead of Vecs for potential performance gain
impl FiniteAutomata {
    fn from_regex_internal(regex: &RegEx, state_transitions: &mut Vec<StateTransitions>) {
        fn kleene_closure(inner: &RegEx, state_transitions: &mut Vec<StateTransitions>) {
            let starting_index = state_transitions.len() - 1;

            // Entry into loop
            state_transitions.push(StateTransitions {
                transitions: vec![]
            });
            let entry_into_loop = state_transitions.len() - 1;

            // Loop
            FiniteAutomata::from_regex_internal(inner, state_transitions);
            let loop_exit = state_transitions.len() - 1;

            // Exit after loop
            state_transitions.push(StateTransitions {
                transitions: vec![]
            });
            let exit_after_loop = state_transitions.len() - 1;

            // Starting index into loop entry
            state_transitions[starting_index].transitions.push(StateTransition {
                condition: None,
                target_state: entry_into_loop
            });
            // Starting index into exit after loop
            state_transitions[starting_index].transitions.push(StateTransition {
                condition: None,
                target_state: exit_after_loop
            });

            // Loop exit into loop entry
            state_transitions[loop_exit].transitions.push(StateTransition {
                condition: None,
                target_state: entry_into_loop
            });
            // Loop exit into exit after loop
            state_transitions[loop_exit].transitions.push(StateTransition {
                condition: None,
                target_state: exit_after_loop
            });
        }

        match regex {
            RegEx::Character(c) => {
                let len = state_transitions.len();
                state_transitions.last_mut().unwrap().transitions.push(StateTransition {
                    condition: Some(*c),
                    target_state: len,
                });
                state_transitions.push(StateTransitions {
                    transitions: vec![]
                });
            }
            RegEx::Concat(lhs, rhs) => {
                Self::from_regex_internal(lhs, state_transitions);
                Self::from_regex_internal(rhs, state_transitions);
            }
            RegEx::KleeneClosure(inner) => {
                kleene_closure(inner, state_transitions);
            }
            RegEx::PositiveClosure(inner) => {
                Self::from_regex_internal(inner, state_transitions);
                kleene_closure(inner, state_transitions);
            }
            RegEx::Union(lhs, rhs) => {
                let starting_index = state_transitions.len() - 1;

                // Entry into lhs
                state_transitions.push(StateTransitions {
                    transitions: vec![]
                });
                let entry_into_lhs = state_transitions.len() - 1;

                Self::from_regex_internal(lhs, state_transitions);
                let lhs_exit = state_transitions.len() - 1;

                // Entry into rhs
                state_transitions.push(StateTransitions {
                    transitions: vec![]
                });
                let entry_into_rhs = state_transitions.len() - 1;

                Self::from_regex_internal(rhs, state_transitions);
                let rhs_exit = state_transitions.len() - 1;

                // Exit after branch
                state_transitions.push(StateTransitions {
                    transitions: vec![]
                });
                let exit_after_branch = state_transitions.len() - 1;

                // Starting index into lhs entry
                state_transitions[starting_index].transitions.push(StateTransition {
                    condition: None,
                    target_state: entry_into_lhs
                });

                // Starting index into rhs entry
                state_transitions[starting_index].transitions.push(StateTransition {
                    condition: None,
                    target_state: entry_into_rhs
                });

                // Lhs exit into exit after branch
                state_transitions[lhs_exit].transitions.push(StateTransition {
                    condition: None,
                    target_state: exit_after_branch
                });

                // Rhs exit into exit after branch
                state_transitions[rhs_exit].transitions.push(StateTransition {
                    condition: None,
                    target_state: exit_after_branch
                });
            }
        }
    }

    pub fn new_from_regex(regex: &RegEx) -> Self {
        let mut state_transitions = vec![];
        // Starting state
        state_transitions.push(StateTransitions {
            transitions: vec![]
        });
        Self::from_regex_internal(regex, &mut state_transitions);
        Self {
            // The last state will be at the last used state index
            accepting_states: vec![state_transitions.len() - 1],
            state_transitions,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            accepting_states: vec![],
            state_transitions: vec![]
        }
    }

    pub fn move_at_set(&self, state_set: &[usize], condition: char, result: &mut Vec<usize>) {
        for state in state_set {
            self.move_at(*state, condition, result);
        }
    }

    pub fn epsilon_closure_at_set(&self, state_set: &[usize], result: &mut Vec<usize>) {
        for state in state_set {
            self.epsilon_closure_at(*state, result);
        }
    }

    pub fn move_at(&self, state: usize, condition: char, result: &mut Vec<usize>) {
        for state_transition in &self.state_transitions[state].transitions {
            if let Some(c) = state_transition.condition {
                if c == condition {
                    if !result.contains(&state_transition.target_state) {
                        result.push(state_transition.target_state);
                    }
                }
            }
        }
    }

    pub fn epsilon_closure_at(&self, state: usize, result: &mut Vec<usize>) {
        if !result.contains(&state) {
            result.push(state)
        }
        for state_transition in &self.state_transitions[state].transitions {
            if let None = state_transition.condition {
                if !result.contains(&state_transition.target_state) {
                    result.push(state_transition.target_state);
                    self.epsilon_closure_at(state_transition.target_state, result);
                }
            }
        }
    }

    pub fn used_alphabet_at(&self, state: usize, alphabet: &mut Vec<char>) {
        for transition in &self.state_transitions[state].transitions {
            if let Some(condition) = transition.condition {
                if !alphabet.contains(&condition) {
                    alphabet.push(condition);
                }
            }
        }
    }

    pub fn nfa_to_dfa(&self) -> FiniteAutomata {
        let mut result = FiniteAutomata::new_empty();
        result.state_transitions.push(StateTransitions {
            transitions: vec![]
        });

        let mut done_result_states: HashMap<Vec<usize>, usize> = HashMap::new();
        let mut will_be_done_result_states: HashMap<Vec<usize>, usize> = HashMap::new();

        fn add_dfa_state_from_nfa_state_set(
            nfa: &FiniteAutomata,
            target_dfa: &mut FiniteAutomata,
            nfa_state_set: &[usize],
            done_result_states: &mut HashMap<Vec<usize>, usize>,
            will_be_done_result_states: &mut HashMap<Vec<usize>, usize>,
            dfa_state_index: usize,
            max_used_state_index: &mut usize
        ) {
            if done_result_states.contains_key(nfa_state_set) {
                return;
            }

            done_result_states.insert(nfa_state_set.to_vec(), dfa_state_index);
            if will_be_done_result_states.contains_key(nfa_state_set) {
                will_be_done_result_states.remove(nfa_state_set);
            }

            for state in nfa_state_set {
                if nfa.accepting_states.contains(state) {
                    target_dfa.accepting_states.push(dfa_state_index);
                    break;
                }
            }

            let mut alphabet = vec![];
            for nfa_state in nfa_state_set {
                nfa.used_alphabet_at(*nfa_state, &mut alphabet);
            }

            let mut transitions: HashMap<char, (Vec<usize>, usize)> = HashMap::new();
            for character in alphabet {
                let mut transition_nfa_set = vec![];
                let mut transition_nfa_set_after_closure = vec![];

                nfa.move_at_set(&nfa_state_set, character, &mut transition_nfa_set);
                nfa.epsilon_closure_at_set(&transition_nfa_set, &mut transition_nfa_set_after_closure);

                transition_nfa_set_after_closure.sort();

                let index = if let Some(index) = done_result_states.get(&transition_nfa_set_after_closure) {
                    *index
                } else if let Some(index) = will_be_done_result_states.get(&transition_nfa_set_after_closure) {
                    *index
                }
                else {
                    // If we haven't done this nfa state set already / nor space is already reserved for it, we reserve space for it inside the target dfa
                    *max_used_state_index += 1;
                    target_dfa.state_transitions.push(StateTransitions {
                        transitions: vec![]
                    });
                    will_be_done_result_states.insert(transition_nfa_set_after_closure.clone(), *max_used_state_index);
                    *max_used_state_index
                };

                transitions.insert(character, (transition_nfa_set_after_closure, index));
            }

            for transition_target in transitions.values() {
                add_dfa_state_from_nfa_state_set(
                    nfa,
                    target_dfa,
                    &transition_target.0,
                    done_result_states,
                    will_be_done_result_states,
                    transition_target.1,
                    max_used_state_index
                );
            }

            target_dfa.state_transitions[dfa_state_index].transitions.append(&mut transitions.iter().map(|t| {
                StateTransition {
                    condition: Some(*t.0),
                    target_state: t.1.1
                }
            }).collect());
        }

        let mut starting_state = vec![];
        self.epsilon_closure_at(0, &mut starting_state);

        add_dfa_state_from_nfa_state_set(
            self,
            &mut result,
            &starting_state,
            &mut done_result_states,
            &mut will_be_done_result_states,
            0,
            &mut 0
        );

        result
    }

    pub fn dump(&self) {
        for i in 0..self.state_transitions.len() {
            print!("{} :\n", i);
            for transition in &self.state_transitions[i].transitions {
                println!("    {} -> {}", if let Some(c) = transition.condition { format!("'{}'", c) } else { "Epsilon".to_owned() }, transition.target_state );
            }
            if self.accepting_states.contains(&i) {
                println!("    (Accepting)");
            }
            if self.state_transitions[i].transitions.is_empty() {
                println!("    (No transitions)")
            }
        }
    }
}