use std::{borrow::Cow, collections::VecDeque};

use thiserror::Error;

use crate::{Attribute, Document, Element, EntityRef};

#[derive(Debug, Error)]
pub enum InsertElementError {
    #[error("couldn't insert into {0} because it wasn't found")]
    NotFound(EntityRef),

    #[error("couldn't replace child {0} in {1} because {1} only has {2} children")]
    ReplaceChildOutOfRange(usize, EntityRef, usize),

    #[error("error while replacing child {0} in {1}: {2}")]
    DropEntityError(usize, EntityRef, DropEntityError),
}

#[derive(Debug, Error)]
pub enum DropEntityError {
    #[error("the root `<document>` tag cannot be dropped")]
    RefuseDropRoot,

    #[error("couldn't drop the {0} because it wasn't found")]
    NotFound(EntityRef),
}

/// The position an item should be placed within its parent.
pub enum PlacePosition {
    /// Insert at the front, pushing others to the back.
    InsertFront,
    /// Insert at the back.
    InsertBack,
    /// Insert at N, capped at the back of the parent.
    InsertFrontN(usize),
    /// Insert at the Nth last position, capped at the front of the parent.
    InsertBackN(usize),
    /// Replace the Nth item.
    Replace(usize),
}

impl<'a> Document<'a> {
    /// Insert a new element into the document.
    pub fn insert<
        C: Into<Cow<'a, str>>,
        C2: Into<Cow<'a, str>>,
        VD: Into<VecDeque<Attribute<'a>>>,
    >(
        &mut self,
        parent_id: usize,
        place_position: PlacePosition,
        prefix: C,
        local: C,
        attributes: VD,
    ) -> Result<(), InsertElementError> {
        self.element_store.push(Some(Element {
            parent: parent_id,
            prefix: prefix.into(),
            local: local.into(),
            attributes: attributes.into(),
            ..Default::default()
        }));

        let element_num = self.element_store.len();

        if self.element_store[parent_id].is_some() {
            let parent_end = self.element_store[parent_id]
                .as_mut()
                .unwrap()
                .children
                .len()
                - 1;
            match place_position {
                PlacePosition::InsertFront => {
                    self.element_store[parent_id]
                        .as_mut()
                        .unwrap()
                        .children
                        .push_front(EntityRef::Element(element_num));
                    Ok(())
                }
                PlacePosition::InsertBack => {
                    self.element_store[parent_id]
                        .as_mut()
                        .unwrap()
                        .children
                        .push_back(EntityRef::Element(element_num));
                    Ok(())
                }
                PlacePosition::InsertFrontN(n) => {
                    self.element_store[parent_id]
                        .as_mut()
                        .unwrap()
                        .children
                        .insert(n.min(parent_end), EntityRef::Element(element_num));

                    Ok(())
                }
                PlacePosition::InsertBackN(n) => {
                    self.element_store[parent_id]
                        .as_mut()
                        .unwrap()
                        .children
                        .insert(
                            parent_end.saturating_sub(n),
                            EntityRef::Element(element_num),
                        );

                    Ok(())
                }
                PlacePosition::Replace(n) => {
                    if n > self.element_store[parent_id]
                        .as_ref()
                        .unwrap()
                        .children
                        .len()
                        - 1
                    {
                        Err(InsertElementError::ReplaceChildOutOfRange(
                            n,
                            EntityRef::Element(parent_id),
                            self.element_store[parent_id]
                                .as_ref()
                                .unwrap()
                                .children
                                .len()
                                - 1,
                        ))
                    } else {
                        self.drop(
                            self.element_store[parent_id].as_ref().unwrap().children[n].clone(),
                        )
                        .map_err(|e| {
                            InsertElementError::DropEntityError(n, EntityRef::Element(parent_id), e)
                        })?;
                        Ok(())
                    }
                }
            }
        } else {
            Err(InsertElementError::NotFound(EntityRef::Element(parent_id)))
        }
    }

    /// Manually drop an entity and its children
    pub fn drop(&mut self, entity_ref: EntityRef) -> Result<(), DropEntityError> {
        self.drop_impl(entity_ref, true)
    }

    // this code may look dirty and disgusting, but it's incredibly fast. that's true beauty, baby
    fn drop_impl(&mut self, entity_ref: EntityRef, is_parent: bool) -> Result<(), DropEntityError> {
        match entity_ref {
            EntityRef::Element(i) => {
                let element = self.element_store.get(i);
                if i == 0 {
                    Err(DropEntityError::RefuseDropRoot)
                } else if element.is_some() {
                    if element.unwrap().is_some() {
                        let children = &self.element_store[i].as_ref().unwrap().children;

                        // any way around this? we get mutability of the whole struct which rust hates
                        // but we know that each child is mutated independently of the parent or themselves
                        for child in children.clone() {
                            self.drop_impl(child, false)?;
                        }

                        if is_parent {
                            let mut sel = None;
                            let parent = self.element_store[i].as_ref().unwrap().parent;

                            for (j, c) in self.element_store[parent]
                                .as_ref()
                                .unwrap()
                                .children
                                .iter()
                                .enumerate()
                            {
                                if c == &entity_ref {
                                    sel = Some(j);
                                    break;
                                }
                            }

                            if let Some(s) = sel {
                                self.element_store[parent]
                                    .as_mut()
                                    .unwrap()
                                    .children
                                    .remove(s)
                                    .unwrap();
                            }
                        }
                    } else {
                        return Err(DropEntityError::NotFound(entity_ref.clone()));
                    }

                    self.element_store[i] = None;

                    Ok(())
                } else {
                    Err(DropEntityError::NotFound(entity_ref.clone()))
                }
            }
            EntityRef::Text(i) => {
                if self.text_store.get(i).is_some() {
                    if self.text_store.get(i).unwrap().is_some() {
                        if is_parent {
                            let mut sel = None;
                            let parent = self.text_store[i].as_ref().unwrap().parent;
                            for (j, c) in self.element_store[parent]
                                .as_mut()
                                .unwrap()
                                .children
                                .iter()
                                .enumerate()
                            {
                                if c == &entity_ref {
                                    sel = Some(j);
                                    break;
                                }
                            }

                            if let Some(s) = sel {
                                self.element_store[parent]
                                    .as_mut()
                                    .unwrap()
                                    .children
                                    .remove(s)
                                    .unwrap();
                            }
                        }

                        self.text_store[i] = None;

                        Ok(())
                    } else {
                        Err(DropEntityError::NotFound(entity_ref.clone()))
                    }
                } else {
                    Err(DropEntityError::NotFound(entity_ref.clone()))
                }
            }
        }
    }
}
