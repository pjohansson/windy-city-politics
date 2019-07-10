use crate::{
    consts::{DONE_KNOT, END_KNOT},
    error::{FollowError, InternalError, ParseError},
    follow::{FollowResult, LineDataBuffer, Next},
    knot::Knot,
};

use std::collections::HashMap;

use super::{
    parse::read_knots_from_string,
    process::{prepare_choices_for_user, process_buffer},
};

#[derive(Clone, Debug)]
/// Single line of text in a story, ready to display.
pub struct Line {
    /// Text content.
    pub text: String,
    /// Tags set to the line.
    pub tags: Vec<String>,
}

/// Convenience type to indicate when a buffer of `Line` objects is being manipulated.
pub type LineBuffer = Vec<Line>;

#[derive(Debug)]
/// Story with knots, diverts, choices and possibly lots of text.
pub struct Story {
    knots: HashMap<String, Knot>,
    stack: Vec<String>,
}

/// Result from following a `Story`.
///
/// # Examples
/// ```
/// # use inkling::{read_story_from_string, StoryAction};
/// let content = "\
/// Professor Lidenbrock had barely a spattering of water left in his flask.
/// *   Axel got the last of it.
/// *   He pressed on, desperately hoping to find water soon.
/// ";
///
/// let mut story = read_story_from_string(content).unwrap();
/// let mut line_buffer = Vec::new();
///
/// match story.start(&mut line_buffer).unwrap() {
///     StoryAction::Choice(choice_set) => {
///         println!("Choose:");
///         for (i, choice) in choice_set.iter().enumerate() {
///             println!("{}. {}", i + 1, choice.text);
///         }
///     },
///     StoryAction::Done => { /* the story reached its end */ },
/// }
/// ```
pub enum StoryAction {
    /// The story reached an end.
    Done,
    /// A choice was encountered. Tags found with the choice are preserved.
    Choice(Vec<Line>),
}

impl StoryAction {
    /// If a set of choices was returned, retrieve them without having to match.
    ///
    /// # Examples
    /// ```
    /// # use inkling::{read_story_from_string, StoryAction};
    /// let content = "\
    /// Professor Lidenbrock had barely a spattering of water left in his flask.
    /// *   Axel got the last of it.
    /// *   He pressed on, desperately hoping to find water soon.
    /// ";
    ///
    /// let mut story = read_story_from_string(content).unwrap();
    /// let mut line_buffer = Vec::new();
    ///
    /// if let Some(choices) = story.start(&mut line_buffer).unwrap().get_choices() {
    ///     /* do what you want */
    /// }
    /// ```
    pub fn get_choices(&self) -> Option<Vec<Line>> {
        match self {
            StoryAction::Choice(choices) => Some(choices.clone()),
            _ => None,
        }
    }
}

impl Story {
    /// Start walking through the story while reading all lines into the supplied buffer.
    /// Returns either when the story reached an end or when a set of choices was encountered,
    /// which requires the user to select one. Continue the story with `resume_with_choice`.
    ///
    /// # Notes
    /// The input line buffer is not cleared before reading new lines into it.
    ///
    /// # Examples
    /// ```
    /// # use inkling::{read_story_from_string, Story};
    /// let content = "\
    /// Only in silence the word,
    /// only in dark the light,
    /// only in dying life:
    /// bright the hawk’s flight
    /// on the empty sky.
    /// ";
    ///
    /// let mut story: Story = read_story_from_string(content).unwrap();
    /// let mut line_buffer = Vec::new();
    ///
    /// story.start(&mut line_buffer);
    ///
    /// assert_eq!(line_buffer.last().unwrap().text, "on the empty sky.\n");
    /// ```
    pub fn start(&mut self, line_buffer: &mut LineBuffer) -> Result<StoryAction, FollowError> {
        Self::follow_story_wrapper(
            self,
            |_self, buffer| Self::follow_knot(_self, buffer),
            line_buffer,
        )
    }

    /// Resume the story with the choice corresponding to the input `index`. Indexing starts
    /// from 0, so the third choice in a set will have index 2.
    ///
    /// The story continues until it reaches a dead end or another set of choices
    /// is encountered.
    ///
    /// # Notes
    /// The input line buffer is not cleared before reading new lines into it.
    /// # Examples
    /// ```
    /// # use inkling::read_story_from_string;
    /// let content = "\
    /// Just as Nancy picked the old diary up from the table she heard
    /// the door behind her creak open. Someone’s coming!
    ///
    /// *   She spun around to face the danger head on.
    ///     Her heart was racing as the door slowly swung open and the black
    ///     cat from the garden swept in.
    ///     “Miao!”   
    /// *   In one smooth motion she hid behind the large curtain.
    ///     A familiar “meow” coming from the room filled her with relief.
    ///     That barely lasted a moment before the dusty curtains made her
    ///     sneeze, awakening the house keeper sleeping in the neighbouring room.
    /// ";
    ///
    /// let mut story = read_story_from_string(content).unwrap();
    /// let mut line_buffer = Vec::new();
    ///
    /// story.start(&mut line_buffer);
    /// story.resume_with_choice(0, &mut line_buffer);
    ///
    /// assert_eq!(line_buffer.last().unwrap().text, "“Miao!”\n");
    /// ```
    pub fn resume_with_choice(
        &mut self,
        index: usize,
        line_buffer: &mut LineBuffer,
    ) -> Result<StoryAction, FollowError> {
        Self::follow_story_wrapper(
            self,
            |_self, buffer| Self::follow_knot_with_choice(_self, index, buffer),
            line_buffer,
        )
    }

    /// Wrapper of common behavior between `start` and `resume_with_choice`. Sets up
    /// a `LineDataBuffer`, reads data into it with the supplied closure and processes
    /// the data by calling `prepare_buffer` on it. If a choice was encountered it
    /// is prepared and returned.
    fn follow_story_wrapper<F>(
        &mut self,
        func: F,
        line_buffer: &mut LineBuffer,
    ) -> Result<StoryAction, FollowError>
    where
        F: FnOnce(&mut Self, &mut LineDataBuffer) -> Result<Next, FollowError>,
    {
        let mut internal_buffer = Vec::new();
        let result = func(self, &mut internal_buffer)?;

        process_buffer(line_buffer, internal_buffer);

        match result {
            Next::ChoiceSet(choice_set) => {
                let user_choice_lines = prepare_choices_for_user(&choice_set);
                Ok(StoryAction::Choice(user_choice_lines))
            }
            Next::Done => Ok(StoryAction::Done),
            Next::Divert(..) => unreachable!("diverts are treated in the closure"),
        }
    }

    /* Internal functions to walk through the story into a `LineDataBuffer`
     * which will be processed into the user supplied lines by the public functions */

    fn follow_knot(&mut self, line_buffer: &mut LineDataBuffer) -> FollowResult {
        self.follow_on_knot_wrapper(|knot, buffer| knot.follow(buffer), line_buffer)
    }

    fn follow_knot_with_choice(
        &mut self,
        choice_index: usize,
        line_buffer: &mut LineDataBuffer,
    ) -> FollowResult {
        self.follow_on_knot_wrapper(
            |knot, buffer| knot.follow_with_choice(choice_index, buffer),
            line_buffer,
        )
    }

    /// Wrapper for common behavior between `follow_knot` and `follow_knot_with_choice`.
    /// Deals with `Diverts` when they are encountered, they are not returned further up
    /// in the call stack.
    fn follow_on_knot_wrapper<F>(&mut self, f: F, buffer: &mut LineDataBuffer) -> FollowResult
    where
        F: FnOnce(&mut Knot, &mut LineDataBuffer) -> FollowResult,
    {
        let knot_name = self.stack.last().unwrap();

        let result = self
            .knots
            .get_mut(knot_name)
            .ok_or(
                InternalError::UnknownKnot {
                    name: knot_name.clone(),
                }
                .into(),
            )
            .and_then(|knot| f(knot, buffer))?;

        match result {
            Next::Divert(to_knot) => {
                if &to_knot == DONE_KNOT || &to_knot == END_KNOT {
                    Ok(Next::Done)
                } else {
                    self.stack.last_mut().map(|knot_name| *knot_name = to_knot);

                    self.follow_knot(buffer)
                }
            }
            _ => Ok(result),
        }
    }
}

/// Read a `Story` by parsing an input string.
///
/// # Examples
/// ```
/// # use inkling::{read_story_from_string, Story};
/// let content = "\
/// He drifted off, and when he opened his eyes the woman was still there.
/// Now she was talking to the old man seated next to her—the farmer from two stations back.
/// ";
///
/// let story: Story = read_story_from_string(content).unwrap();
/// ```
pub fn read_story_from_string(string: &str) -> Result<Story, ParseError> {
    let (root, knots) = read_knots_from_string(string)?;

    Ok(Story {
        knots,
        stack: vec![root],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::str::FromStr;

    #[test]
    fn story_internally_follows_through_knots_when_diverts_are_found() {
        let knot1_name = "back_in_london".to_string();
        let knot2_name = "hurry_home".to_string();

        let knot1_text = format!(
            "\
We arrived into London at 9.45pm exactly.
-> {}\
",
            knot2_name
        );

        let knot2_text = format!(
            "\
             We hurried home to Savile Row as fast as we could.\
             "
        );

        let mut knots = HashMap::new();

        knots.insert(knot1_name.clone(), Knot::from_str(&knot1_text).unwrap());
        knots.insert(knot2_name, Knot::from_str(&knot2_text).unwrap());

        let mut story = Story {
            knots,
            stack: vec![knot1_name],
        };

        let mut buffer = Vec::new();

        story.follow_knot(&mut buffer).unwrap();

        assert_eq!(&buffer.last().unwrap().text, &knot2_text);
    }

    #[test]
    fn story_internally_resumes_from_the_new_knot_after_a_choice_is_made() {
        let knot1_name = "back_in_london".to_string();
        let knot2_name = "hurry_home".to_string();

        let knot1_text = format!(
            "\
We arrived into London at 9.45pm exactly.
-> {}\
",
            knot2_name
        );

        let knot2_text = format!(
            "\
\"What's that?\" my master asked.
*	\"I am somewhat tired[.\"],\" I repeated.
	\"Really,\" he responded. \"How deleterious.\"
*	\"Nothing, Monsieur!\"[] I replied.
	\"Very good, then.\"
*   \"I said, this journey is appalling[.\"] and I want no more of it.\"
	\"Ah,\" he replied, not unkindly. \"I see you are feeling frustrated. Tomorrow, things will improve.\"\
"
        );

        let mut knots = HashMap::new();

        knots.insert(knot1_name.clone(), Knot::from_str(&knot1_text).unwrap());
        knots.insert(knot2_name, Knot::from_str(&knot2_text).unwrap());

        let mut story = Story {
            knots,
            stack: vec![knot1_name],
        };

        let mut buffer = Vec::new();

        story.follow_knot(&mut buffer).unwrap();
        story.follow_knot_with_choice(1, &mut buffer).unwrap();

        assert_eq!(&buffer.last().unwrap().text, "\"Very good, then.\"");
    }

    #[test]
    fn when_a_knot_is_returned_to_the_text_starts_from_the_beginning() {
        let knot1_name = "back_in_london".to_string();
        let knot2_name = "hurry_home".to_string();

        let knot1_line = "We arrived into London at 9.45pm exactly.";

        let knot1_text = format!(
            "\
{}
-> {}\
",
            knot1_line, knot2_name
        );

        let knot2_text = format!(
            "\
*   We hurried home to Savile Row as fast as we could. 
*   But we decided our trip wasn't done and immediately left.
    After a few days me returned again.
    -> {}\
",
            knot1_name
        );

        let mut knots = HashMap::new();

        knots.insert(knot1_name.clone(), Knot::from_str(&knot1_text).unwrap());
        knots.insert(knot2_name, Knot::from_str(&knot2_text).unwrap());

        let mut story = Story {
            knots,
            stack: vec![knot1_name],
        };

        let mut buffer = Vec::new();

        story.follow_knot(&mut buffer).unwrap();
        story.follow_knot_with_choice(1, &mut buffer).unwrap();

        assert_eq!(&buffer[0].text, knot1_line);
        assert_eq!(&buffer[5].text, knot1_line);
    }

    #[test]
    fn divert_to_done_or_end_constant_knots_ends_story() {
        let knot_done_text = "\
    -> DONE
    ";

        let knot_end_text = "\
    -> END
    ";

        let knot_done = Knot::from_str(&knot_done_text).unwrap();
        let knot_end = Knot::from_str(&knot_end_text).unwrap();

        let mut knots = HashMap::new();
        knots.insert("knot_done".to_string(), knot_done);
        knots.insert("knot_end".to_string(), knot_end);

        let mut story = Story {
            knots,
            stack: vec!["knot_done".to_string()],
        };

        let mut buffer = Vec::new();

        match story.start(&mut buffer).unwrap() {
            StoryAction::Done => (),
            _ => panic!("story should be done when diverting to DONE knot"),
        }

        story.stack = vec!["knot_end".to_string()];

        match story.start(&mut buffer).unwrap() {
            StoryAction::Done => (),
            _ => panic!("story should be done when diverting to END knot"),
        }
    }
}