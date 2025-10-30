use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{PropValue, ScreenGraph, WidgetNode};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MergeConflict {
    pub path: String,
    pub base: Option<serde_json::Value>,
    pub left: Option<serde_json::Value>,
    pub right: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MergeOutcome {
    pub screen: ScreenGraph,
    pub conflicts: Vec<MergeConflict>,
}

pub fn merge_screen_graphs(
    base: &ScreenGraph,
    left: &ScreenGraph,
    right: &ScreenGraph,
) -> MergeOutcome {
    let mut conflicts = Vec::new();
    let merged_id = merge_scalar("screen.id", &base.id, &left.id, &right.id, &mut conflicts);

    if left.id != right.id && (left.id == base.id || right.id == base.id) {
        record_conflict(
            "screen.id",
            Some(&base.id),
            Some(&left.id),
            Some(&right.id),
            &mut conflicts,
        );
    }

    let merged_root = merge_widget_node(
        "screen.root",
        &base.root,
        &left.root,
        &right.root,
        &mut conflicts,
    );

    MergeOutcome {
        screen: ScreenGraph {
            id: merged_id,
            root: merged_root,
        },
        conflicts,
    }
}

fn merge_widget_node(
    path: &str,
    base: &WidgetNode,
    left: &WidgetNode,
    right: &WidgetNode,
    conflicts: &mut Vec<MergeConflict>,
) -> WidgetNode {
    let widget = merge_scalar(
        &format!("{path}.widget"),
        &base.widget,
        &left.widget,
        &right.widget,
        conflicts,
    );

    let props = merge_props(
        &format!("{path}.props"),
        &base.props,
        &left.props,
        &right.props,
        conflicts,
    );

    let children = merge_children(
        &format!("{path}.children"),
        &base.children,
        &left.children,
        &right.children,
        conflicts,
    );

    WidgetNode {
        widget,
        props,
        children,
    }
}

fn merge_props(
    path: &str,
    base: &BTreeMap<String, PropValue>,
    left: &BTreeMap<String, PropValue>,
    right: &BTreeMap<String, PropValue>,
    conflicts: &mut Vec<MergeConflict>,
) -> BTreeMap<String, PropValue> {
    let mut keys = BTreeSet::new();
    for key in base.keys().chain(left.keys()).chain(right.keys()) {
        keys.insert(key);
    }

    let mut merged = BTreeMap::new();
    for key in keys {
        let base_val = base.get(key);
        let left_val = left.get(key);
        let right_val = right.get(key);

        let merged_val = merge_optional(
            &format!("{path}.{key}"),
            base_val,
            left_val,
            right_val,
            conflicts,
        );

        if let Some(value) = merged_val {
            merged.insert(key.clone(), value);
        }
    }

    merged
}

fn merge_children(
    path: &str,
    base: &[WidgetNode],
    left: &[WidgetNode],
    right: &[WidgetNode],
    conflicts: &mut Vec<MergeConflict>,
) -> Vec<WidgetNode> {
    if left == right {
        return left.to_vec();
    }
    if left == base {
        return right.to_vec();
    }
    if right == base {
        return left.to_vec();
    }

    let max_len = base.len().max(left.len()).max(right.len());
    let mut merged = Vec::with_capacity(max_len);

    for idx in 0..max_len {
        let child_path = format!("{path}[{idx}]");
        let base_child = base.get(idx);
        let left_child = left.get(idx);
        let right_child = right.get(idx);

        match (base_child, left_child, right_child) {
            (Some(b), Some(l), Some(r)) => {
                merged.push(merge_widget_node(&child_path, b, l, r, conflicts));
            }
            (None, Some(l), Some(r)) => {
                if l == r {
                    merged.push(l.clone());
                } else {
                    record_conflict(&child_path, None, Some(l), Some(r), conflicts);
                    merged.push(r.clone());
                }
            }
            (Some(b), Some(l), None) => {
                record_conflict(&child_path, Some(b), Some(l), None, conflicts);
                // analyzer deleted the child; keep analyzer decision by skipping
            }
            (Some(b), None, Some(r)) => {
                record_conflict(&child_path, Some(b), None, Some(r), conflicts);
                merged.push(r.clone());
            }
            (None, Some(l), None) => {
                merged.push(l.clone());
            }
            (None, None, Some(r)) => {
                merged.push(r.clone());
            }
            (Some(_), None, None) | (None, None, None) => {
                // node absent in merge; nothing to push
            }
        }
    }

    merged
}

fn merge_scalar<T>(
    path: &str,
    base: &T,
    left: &T,
    right: &T,
    conflicts: &mut Vec<MergeConflict>,
) -> T
where
    T: PartialEq + Clone + Serialize,
{
    if left == right {
        return left.clone();
    }
    if left == base {
        return right.clone();
    }
    if right == base {
        return left.clone();
    }

    record_conflict(path, Some(base), Some(left), Some(right), conflicts);
    right.clone()
}

fn merge_optional<T>(
    path: &str,
    base: Option<&T>,
    left: Option<&T>,
    right: Option<&T>,
    conflicts: &mut Vec<MergeConflict>,
) -> Option<T>
where
    T: PartialEq + Clone + Serialize,
{
    if left == right {
        return left.cloned();
    }
    if left == base {
        return right.cloned();
    }
    if right == base {
        return left.cloned();
    }

    if left.is_none() && right.is_none() {
        return None;
    }

    record_conflict(path, base, left, right, conflicts);
    right.cloned()
}

fn record_conflict<T>(
    path: &str,
    base: Option<&T>,
    left: Option<&T>,
    right: Option<&T>,
    conflicts: &mut Vec<MergeConflict>,
) where
    T: Serialize,
{
    conflicts.push(MergeConflict {
        path: path.to_string(),
        base: base.and_then(to_json),
        left: left.and_then(to_json),
        right: right.and_then(to_json),
    });
}

fn to_json<T: Serialize>(value: &T) -> Option<serde_json::Value> {
    serde_json::to_value(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    fn widget(widget: &str, props: &[(&str, PropValue)], children: Vec<WidgetNode>) -> WidgetNode {
        let mut map = BTreeMap::new();
        for (key, value) in props {
            map.insert((*key).to_string(), value.clone());
        }
        WidgetNode {
            widget: widget.to_string(),
            props: map,
            children,
        }
    }

    fn literal(value: &str) -> PropValue {
        PropValue::Literal {
            value: serde_json::Value::String(value.to_string()),
        }
    }

    #[test]
    fn merges_disjoint_prop_changes_without_conflict() {
        let base = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[("value", literal("Hello"))], vec![]),
        };
        let left = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget(
                "Text",
                &[("value", literal("Hello")), ("color", literal("red"))],
                vec![],
            ),
        };
        let right = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[("value", literal("Hi"))], vec![]),
        };

        let outcome = merge_screen_graphs(&base, &left, &right);
        assert!(outcome.conflicts.is_empty());
        assert_eq!(
            outcome.screen.root.props.get("color"),
            Some(&literal("red"))
        );
        assert_eq!(outcome.screen.root.props.get("value"), Some(&literal("Hi")));
    }

    #[test]
    fn records_conflict_for_same_prop_change() {
        let base = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[("value", literal("Hello"))], vec![]),
        };
        let left = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[("value", literal("Hello left"))], vec![]),
        };
        let right = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[("value", literal("Hello right"))], vec![]),
        };

        let outcome = merge_screen_graphs(&base, &left, &right);
        assert_eq!(outcome.conflicts.len(), 1);
        assert_eq!(outcome.conflicts[0].path, "screen.root.props.value");
        assert_eq!(
            outcome.screen.root.props.get("value"),
            Some(&literal("Hello right"))
        );
        assert_eq!(
            outcome.conflicts[0].left,
            Some(json!({ "type": "literal", "value": "Hello left" }))
        );
        assert_eq!(
            outcome.conflicts[0].right,
            Some(json!({ "type": "literal", "value": "Hello right" }))
        );
    }

    #[test]
    fn detects_delete_vs_modify_conflict_in_children() {
        let base_child = widget("Text", &[("value", literal("Hello"))], vec![]);
        let base = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Column", &[], vec![base_child]),
        };

        let left = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Column", &[], vec![]),
        };

        let right_child = widget("Text", &[("value", literal("Updated"))], vec![]);
        let right = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Column", &[], vec![right_child]),
        };

        let outcome = merge_screen_graphs(&base, &left, &right);
        assert_eq!(outcome.conflicts.len(), 1);
        assert_eq!(outcome.conflicts[0].path, "screen.root.children[0]");
        assert_eq!(outcome.screen.root.children.len(), 1);
        assert_eq!(
            outcome.screen.root.children[0].props.get("value"),
            Some(&literal("Updated"))
        );
    }

    #[test]
    fn merges_added_children_with_conflict_entry() {
        let base = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Column", &[], vec![]),
        };

        let left = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget(
                "Column",
                &[],
                vec![widget("Text", &[("value", literal("Left"))], vec![])],
            ),
        };

        let right = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget(
                "Column",
                &[],
                vec![widget("Text", &[("value", literal("Right"))], vec![])],
            ),
        };

        let outcome = merge_screen_graphs(&base, &left, &right);
        assert_eq!(outcome.conflicts.len(), 1);
        assert_eq!(outcome.conflicts[0].path, "screen.root.children[0]");
        assert_eq!(
            outcome.screen.root.children[0].props.get("value"),
            Some(&literal("Right"))
        );
    }

    #[test]
    fn id_conflicts_are_reported() {
        let base = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[], vec![]),
        };
        let left = ScreenGraph {
            id: "Dashboard".to_string(),
            root: widget("Text", &[], vec![]),
        };
        let right = ScreenGraph {
            id: "AnalyzerDashboard".to_string(),
            root: widget("Text", &[], vec![]),
        };

        let outcome = merge_screen_graphs(&base, &left, &right);
        assert_eq!(outcome.conflicts.len(), 1);
        assert_eq!(outcome.conflicts[0].path, "screen.id");
        assert_eq!(outcome.screen.id, "AnalyzerDashboard");
    }
}
