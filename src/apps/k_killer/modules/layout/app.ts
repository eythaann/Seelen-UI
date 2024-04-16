import { current } from '@reduxjs/toolkit';

import { BoxSubType, BoxType, Container, Layout } from './domain';

export function addHandleToLayout(layout: Layout, handle: number): void {
  if (hasSpace(layout.structure)) {
    addHandleToContainer(layout.structure, handle);
    return;
  }
  layout.floating.push(handle);
}

function addHandleToContainer(container: Container, handle: number): void {
  if (container.type === BoxType.Stack) {
    if (!container.handles.length) {
      container.active = handle;
    }
    container.handles.push(handle);
    return;
  }

  if (container.type === BoxType.Reserved && !container.handle) {
    container.handle = handle;
    return;
  }

  if (container.type === BoxType.Horizontal || container.type === BoxType.Vertical) {
    const sortedByPriority = [...container.children].sort((a, b) => a.priority - b.priority);
    for (const child of sortedByPriority) {
      if (hasSpace(child)) {
        addHandleToContainer(child, handle);
        break;
      }
    }
  }
}

export function removeHandleFromLayout(layout: Layout, handle: number): void {
  let floatingIndex = layout.floating.indexOf(handle);
  if (floatingIndex !== -1) {
    layout.floating.splice(floatingIndex, 1);
    return;
  }
  if (removeHandleFromContainer(layout.structure, handle)) {
    reIndexContainer(layout.structure);
  };
}

function removeHandleFromContainer(container: Container, handle: number): boolean {
  if (container.type === BoxType.Stack) {
    const index = container.handles.indexOf(handle);
    if (index !== -1) {
      container.handles.splice(index, 1);
      if (handle === container.active) {
        container.active = container.handles[0] || null;
      }
      return true;
    }
    return false;
  }

  if (container.type === BoxType.Reserved && container.handle === handle) {
    container.handle = undefined;
    return true;
  }

  if (container.type === BoxType.Horizontal || container.type === BoxType.Vertical) {
    for (let index = 0; index < container.children.length; index++) {
      const child = container.children[index]!;
      if (removeHandleFromContainer(child, handle)) {
        /*         if (isEmptyContainer(child) && child.subtype === BoxSubType.Temporal) {
          container.children.splice(index, 1);
        } */
        return true;
      };
    }
  }

  return false;
}

function clearContainer(container: Container): number[] {
  const deleted: number[] = [];
  switch (container.type) {
    case BoxType.Stack:
      deleted.push(...container.handles);
      container.handles = [];
      container.active = null;
      break;

    case BoxType.Reserved:
      if (container.handle) {
        deleted.push(container.handle);
        container.handle = undefined;
      }
      break;

    case BoxType.Horizontal:
    case BoxType.Vertical:
      const sortedByPriority = [...container.children].sort((a, b) => a.priority - b.priority);
      for (const child of sortedByPriority) {
        deleted.push(...clearContainer(child));
      }
      break;

    default:
      console.error('Unknown container type');
  }
  return deleted;
}

function reIndexContainer(container: Container): void {
  const handlesToRestore: number[] = clearContainer(container);
  handlesToRestore.forEach((handle) => addHandleToContainer(container, handle));
}

function hasSpace(container: Container): boolean {
  if (container.type === BoxType.Stack) {
    return true;
  }

  if (container.type === BoxType.Reserved) {
    return !container.handle;
  }

  if (container.type === BoxType.Horizontal || container.type === BoxType.Vertical) {
    return container.children.some(hasSpace);
  }

  return false;
}

export function isEmptyContainer(container: Container): boolean {
  if (container.type === BoxType.Stack) {
    return container.handles.length === 0;
  }

  if (container.type === BoxType.Reserved) {
    return !container.handle;
  }

  if (container.type === BoxType.Horizontal || container.type === BoxType.Vertical) {
    return container.children.every(isEmptyContainer);
  }

  return true;
}