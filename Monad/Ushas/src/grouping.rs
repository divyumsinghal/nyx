//! Notification aggregation — group multiple actor events into a single message.

/// Format a notification body grouping multiple actors for the same event.
///
/// # Examples
///
/// ```
/// use ushas::grouping::format_grouped_body;
/// let msg = format_grouped_body("post.liked", &["Alice".to_string()]);
/// assert_eq!(msg, "Alice liked your post");
/// ```
#[must_use]
pub fn format_grouped_body(event_type: &str, actor_names: &[String]) -> String {
    let action = match event_type {
        "post.liked" => "liked your post",
        "comment.created" => "commented on your post",
        "user.followed" => "followed you",
        _ => "interacted with your content",
    };
    match actor_names {
        [] => String::new(),
        [a] => format!("{a} {action}"),
        [a, b] => format!("{a} and {b} {action}"),
        [a, rest @ ..] => format!("{a} and {} others {action}", rest.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_actor() {
        assert_eq!(
            format_grouped_body("post.liked", &["Alice".into()]),
            "Alice liked your post"
        );
    }

    #[test]
    fn two_actors() {
        assert_eq!(
            format_grouped_body("post.liked", &["Alice".into(), "Bob".into()]),
            "Alice and Bob liked your post"
        );
    }

    #[test]
    fn many_actors() {
        let names: Vec<_> = (0..10).map(|i| format!("User{i}")).collect();
        let r = format_grouped_body("post.liked", &names);
        assert!(r.contains("User0") && r.contains("9 others"));
    }

    #[test]
    fn empty() {
        assert!(format_grouped_body("post.liked", &[]).is_empty());
    }

    #[test]
    fn unknown_event() {
        assert_eq!(
            format_grouped_body("unknown.event", &["Bob".into()]),
            "Bob interacted with your content"
        );
    }

    #[test]
    fn comment_event() {
        assert_eq!(
            format_grouped_body("comment.created", &["Carol".into()]),
            "Carol commented on your post"
        );
    }

    #[test]
    fn follow_event() {
        assert_eq!(
            format_grouped_body("user.followed", &["Dave".into()]),
            "Dave followed you"
        );
    }
}
