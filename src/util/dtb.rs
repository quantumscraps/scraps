use fdt_rs::{
    base::{DevTree, DevTreeNode},
    index::{DevTreeIndex, DevTreeIndexItem},
    prelude::*,
};

/// Looks up a DTB index entry by path
pub fn lookup_dtb_index_entry<'dt>(
    dtb: &'dt DevTreeIndex,
    path: &str,
) -> Option<DevTreeIndexItem<'dt, 'dt, 'dt>> {
    // remove root
    let path = path.trim_start_matches('/');
    let mut current_node = dtb.root();
    let mut prop = None;
    let mut consumed = 0;
    let mut len = 0;

    for component in path.split('/') {
        len += 1;
        for child in current_node.children() {
            if child.name() == Ok(component) {
                current_node = child;
                consumed += 1;
                continue;
            }
        }
        // if we are here there are no matching children
        // so check props instead
        for prop2 in current_node.props() {
            if prop2.name() == Ok(component) {
                prop = Some(prop2);
                consumed += 1;
                // properties are leaves, break
                break;
            }
        }
    }

    // Check if we consumed all components
    if consumed == len {
        Some(if let Some(prop) = prop {
            DevTreeIndexItem::Prop(prop)
        } else {
            DevTreeIndexItem::Node(current_node)
        })
    } else {
        None
    }
}

/// Looks up a DTB entry by path.
///
/// Only supports node leafs and not prop leafs.
pub fn lookup_dtb_entry_node<'a: 'dt, 'path, 'dt>(
    dtb: &'dt DevTree,
    path: &'path str,
) -> Option<DevTreeNode<'a, 'dt>> {
    let path = path.trim_start_matches('/');
    let mut level = 0;
    let len = path.split('/').count();

    for iter_node in dtb.nodes().iterator() {
        let iter_node = iter_node.ok()?;
        // FIXME: This is inefficient
        let path_component = path.split('/').nth(level)?;
        if iter_node.name().ok()? == path_component {
            level += 1;
            if level == len {
                return Some(iter_node);
            }
        }
    }

    None
}

// /// Looks up a DTB entry by path.
// pub fn lookup_dtb_entry<'a: 'dt, 'dt>(
//     dtb: &'dt DevTree,
//     path: &str,
// ) -> Option<(Option<DevTreeNode<'dt, 'dt>>, Option<DevTreeItem<'a, 'dt>>)> {
//     let path = path.trim_start_matches('/');
//     let mut level = 0;
//     let len = path.split('/').count();
//     let mut ret: Option<(Option<DevTreeNode<'dt, 'dt>>, Option<DevTreeItem<'a, 'dt>>)> =
//         Some((None, None));
//     // let mut node2 = None;
//     // let mut prop2 = None;

//     for iter_node in dtb.nodes().iterator() {
//         let iter_node = match iter_node {
//             Ok(n) => n,
//             _ => continue,
//         };

//         // Get nth of path, but this is really inefficient :|
//         let path_component = match path.split('/').nth(level) {
//             Some(c) => c,
//             _ => continue,
//         };
//         if Ok(path_component) == iter_node.name() {
//             // got it!
//             level += 1;
//             // leaf
//             if level == len {
//                 return Some((None, Some(DevTreeItem::Node(iter_node))));
//             }
//             continue;
//         }
//         // Otherwise, check props
//         {
//             match ret {
//                 Some((ref mut n, _)) => *n = Some(iter_node),
//                 _ => {}
//             }
//         }
//         // ret.as_mut().unwrap().0 = Some(iter_node);
//         let prop = 'proplookup: {
//             let node = match ret {
//                 Some((Some(ref n), _)) => n,
//                 _ => break 'proplookup None,
//             };
//             for prop in node.props().iterator() {
//                 let prop = match prop {
//                     Ok(p) => p,
//                     _ => continue,
//                 };

//                 // only allow props at leaf
//                 if Ok(path_component) == prop.name() && level == len - 1 {
//                     level += 1;
//                     break 'proplookup Some(prop);
//                     // ret.as_mut().unwrap().1 = Some(DevTreeItem::Prop(prop));
//                     // return Some(DevTreeItem::Prop(prop.clone()));
//                     // node2 = Some(iter_node.clone());
//                     // prop2 = Some(prop.clone());
//                     // break;
//                 }
//             }
//             None
//         };
//         match ret {
//             Some((_, ref mut thing)) => *thing = prop.map(DevTreeItem::Prop),
//             _ => {}
//         }
//         // ret.unwrap().1 = prop.map(DevTreeItem::Prop);
//     }

//     if ret.as_ref().unwrap().0.is_some() {
//         ret
//     } else {
//         None
//     }
//     // if level == len {
//     //     Some((Some(node2.unwrap()), DevTreeItem::Prop(prop2.unwrap())))
//     // } else {
//     //     None
//     // }
// }

// /// Looks up a DTB entry by path.
// /// Does not support looking up leaf properties.
// ///
// /// Mainly a helper function for the [`lookup_dtb_entry`] macro.
// pub fn lookup_dtb_entry2<'a: 'dt, 'dt>(
//     dtb: &'dt DevTree,
//     path: &str,
//     lookup_leaf: bool,
// ) -> Option<(Option<DevTreeNode<'dt, 'dt>>, Option<DevTreeItem<'a, 'dt>>)> {
//     let path = path.trim_start_matches('/');
//     let mut level = 0;
//     let len = path.split('/').count();
//     let mut ret: Option<(Option<DevTreeNode<'dt, 'dt>>, Option<DevTreeItem<'a, 'dt>>)> =
//         Some((None, None));
//     // let mut node2 = None;
//     // let mut prop2 = None;

//     for iter_node in dtb.nodes().iterator() {
//         let iter_node = match iter_node {
//             Ok(n) => n,
//             _ => continue,
//         };

//         // Get nth of path, but this is really inefficient :|
//         let path_component = match path.split('/').nth(level) {
//             Some(c) => c,
//             _ => continue,
//         };
//         if Ok(path_component) == iter_node.name() {
//             // got it!
//             level += 1;
//             // leaf
//             if level == len {
//                 return Some((None, Some(DevTreeItem::Node(iter_node))));
//             }
//             continue;
//         }
//         // Otherwise, check props
//         ret.unwrap().0 = Some(iter_node);
//         for prop in ret.as_ref().unwrap().0.unwrap().props().iterator() {
//             let prop = match prop {
//                 Ok(p) => p,
//                 _ => continue,
//             };

//             // only allow props at leaf
//             if Ok(path_component) == prop.name() && level == len - 1 {
//                 level += 1;
//                 ret.unwrap().1 = Some(DevTreeItem::Prop(prop));
//                 // return Some(DevTreeItem::Prop(prop.clone()));
//                 // node2 = Some(iter_node.clone());
//                 // prop2 = Some(prop.clone());
//                 // break;
//             }
//         }
//     }

//     if ret.unwrap().1.is_some() {
//         ret
//     } else {
//         None
//     }
//     // if level == len {
//     //     Some((Some(node2.unwrap()), DevTreeItem::Prop(prop2.unwrap())))
//     // } else {
//     //     None
//     // }
// }

// /// Looks up a DTB entry by path.
// ///
// /// Supports leaf properties.
// #[macro_export]
// macro_rules! lookup_dtb_entry {
//     ($dtb:expr, $path:expr) => {
//         'lookup: {
//             let __path = $path.trim_start_matches('/');
//             let mut __level = 0;
//             let __len = __path.split('/').count();
//             // let mut __node = None;
//             // let mut __prop = None;

//             let __possible_proplookup = 'nodelookup: {
//                 for iter_node in $dtb.nodes().iterator() {
//                     let iter_node = match iter_node {
//                         Ok(n) => n,
//                         _ => continue,
//                     };

//                     // Get nth of path, but this is really inefficient :|
//                     let path_component = match __path.split('/').nth(__level) {
//                         Some(c) => c,
//                         _ => continue,
//                     };
//                     if Ok(path_component) == iter_node.name() {
//                         // got it!
//                         __level += 1;
//                         // leaf
//                         if __level == __len {
//                             break 'lookup Some(DevTreeItem::Node(iter_node));
//                         }
//                         continue;
//                     }
//                     // Otherwise, check props
//                     for prop in iter_node.props().iterator() {
//                         let prop = match prop {
//                             Ok(p) => p,
//                             _ => continue,
//                         };

//                         // only allow props at leaf
//                         if Ok(path_component) == prop.name() && __level == __len - 1 {
//                             // Return the node so we can re-run this lookup :\
//                             __level += 1;
//                             break 'nodelookup Some((iter_node, path_component));
//                             // __node = Some(DevTreeItem::Node(core::mem::replace(
//                             //     &mut iter_node,
//                             //     iter_node.clone(),
//                             // )));
//                             // __prop = Some(DevTreeItem::Prop(prop.clone()));
//                             // node2 = Some(iter_node.clone());
//                             // prop2 = Some(prop.clone());
//                             // break;
//                         }
//                     }
//                 }
//                 None
//             };
//             if let Some((node, component)) = __possible_proplookup {
//                 for prop in node.props().iterator() {
//                     let prop = match prop {
//                         Ok(p) => p,
//                         _ => continue,
//                     };

//                     // no need for level check since already done
//                     if Ok(component) == prop.name() {
//                         break 'lookup Some((Some(node), DevTreeItem::Prop(prop)));
//                     }
//                 }
//             }

//             None
//             // if level == len {
//             //     Some((Some(node2.unwrap()), DevTreeItem::Prop(prop2.unwrap())))
//             // } else {
//             //     None
//             // }
//         }
//     };
// }
