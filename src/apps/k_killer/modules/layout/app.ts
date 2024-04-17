import { current } from '@reduxjs/toolkit';
import { invoke } from '@tauri-apps/api/core';
import { clone } from 'lodash';

import { HWND } from '../shared/utils/domain';
import {
  BranchNode,
  FallbackNode,
  Layout,
  LeafNode,
  Node,
  NodeSubtype,
  NodeType,
  Reservation,
} from './domain';

export function removeHandleFromLayout(layout: Layout, handle: number): void {
  let floatingIndex = layout.floating.indexOf(handle);
  if (floatingIndex !== -1) {
    layout.floating.splice(floatingIndex, 1);
    return;
  }
  if (removeHandleFromContainer(layout.structure, handle)) {
    reIndexContainer(layout.structure);
  }
}

function removeHandleFromContainer(container: Node, handle: number): boolean {
  if (container.type === NodeType.Fallback) {
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

  if (container.type === NodeType.Leaf && container.handle === handle) {
    container.handle = null;
    return true;
  }

  if (container.type === NodeType.Horizontal || container.type === NodeType.Vertical) {
    for (let index = 0; index < container.children.length; index++) {
      const child = container.children[index]!;
      if (removeHandleFromContainer(child, handle)) {
        /*         if (isEmptyContainer(child) && child.subtype === BoxSubType.Temporal) {
          container.children.splice(index, 1);
        } */
        return true;
      }
    }
  }

  return false;
}

function clearContainer(container: Node): number[] {
  const deleted: number[] = [];
  switch (container.type) {
    case NodeType.Fallback:
      deleted.push(...container.handles);
      container.handles = [];
      container.active = null;
      break;

    case NodeType.Leaf:
      if (container.handle) {
        deleted.push(container.handle);
        container.handle = null;
      }
      break;

    case NodeType.Horizontal:
    case NodeType.Vertical:
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

function reIndexContainer(container: Node): void {
  const handlesToRestore: number[] = clearContainer(container);
  handlesToRestore.forEach((handle) => NodeImpl.from(container).addHandle(handle));
}

export class NodeImpl<T extends Node> {
  private readonly ref: T;

  private constructor(node: T) {
    this.ref = node;
  }

  static from<T extends Node>(node: T): NodeImpl<T> {
    return new NodeImpl(node);
  }

  /**
   * @param handle the handle of the node
   * @param insertAfter the priority of the node to insert after
   * @returns a new leaf node
   */
  static newLeaf(handle: HWND | null, insertAfter: number = 0): LeafNode {
    return {
      type: NodeType.Leaf,
      subtype: NodeSubtype.Temporal,
      priority: insertAfter + 1,
      handle,
    };
  }

  get inner(): T {
    return this.ref;
  }

  clone_ref(): T {
    return clone(this.ref);
  }

  private unreachable(): never {
    console.error(`Node type ${this.ref.type} is not implemented`);
    throw new Error();
  }

  isLeaf(): this is NodeImpl<LeafNode> {
    return this.ref.type === NodeType.Leaf;
  }

  isFallback(): this is NodeImpl<FallbackNode> {
    return this.ref.type === NodeType.Fallback;
  }

  isBranch(): this is NodeImpl<BranchNode> {
    return this.ref.type === NodeType.Horizontal || this.ref.type === NodeType.Vertical;
  }

  isEmpty(): boolean {
    if (this.isLeaf()) {
      return !this.ref.handle;
    }

    if (this.isFallback()) {
      return this.ref.handles.length === 0;
    }

    if (this.isBranch()) {
      return this.ref.children.every((node) => NodeImpl.from(node).isEmpty());
    }

    this.unreachable();
  }

  isFull(): boolean {
    if (this.isLeaf()) {
      return !!this.ref.handle;
    }

    if (this.isFallback()) {
      // fallback nodes can be fulled this allow infinite number of handles
      return false;
    }

    if (this.isBranch()) {
      return this.ref.children.every((node) => NodeImpl.from(node).isFull());
    }

    this.unreachable();
  }

  addHandle(handle: number): boolean {
    if (this.isLeaf()) {
      this.ref.handle = handle;
      return true;
    }

    if (this.isFallback()) {
      let len = this.ref.handles.push(handle);
      if (len === 1) {
        this.ref.active = handle;
      }
      return true;
    }

    if (this.isBranch()) {
      const sortedByPriority = [...this.ref.children].sort((a, b) => a.priority - b.priority);
      for (const child of sortedByPriority) {
        const node = NodeImpl.from(child);
        if (!node.isFull() && node.addHandle(handle)) {
          return true;
        }
      }
      return false;
    }

    this.unreachable();
  }

  mutateToStacked(): NodeImpl<FallbackNode> {
    if (this.isLeaf()) {
      let ref = this.ref as any;
      ref.type = NodeType.Fallback;
      ref.subtype = NodeSubtype.Temporal;
      ref.handles = [];
      ref.active = null;

      if (this.ref.handle) {
        ref.handles.push(this.ref.handle);
        ref.active = this.ref.handle;
      }

      delete ref.handle;
    }

    if (this.isBranch()) {
      throw new Error('Cannot mutate branch to stacked');
    }

    return this as NodeImpl<FallbackNode>;
  }

  mutateToBranch(type: NodeType.Horizontal | NodeType.Vertical): NodeImpl<BranchNode> {
    if (this.isBranch()) {
      throw new Error('Cannot mutate branch to branch');
    }

    let copy = this.clone_ref();
    let ref = this.ref as any;
    ref.type = type;
    ref.subtype = NodeSubtype.Temporal;
    ref.children = [copy];

    delete ref.handle;
    // TODO(eythan) check priorities for stacked
    delete ref.handles;
    delete ref.active;

    return this as NodeImpl<BranchNode>;
  }

  concreteReservation(hwnd: HWND, reservation: Reservation, activeHandle: number): boolean {
    console.trace(`Reserving ${reservation} for ${hwnd} on ${activeHandle}`);

    const found = this.getNodeContaining(activeHandle);
    if (!found) {
      console.error('Could not find node containing handle', activeHandle);
      return false;
    }

    const node = NodeImpl.from(found);

    switch (reservation) {
      case Reservation.Stack: {
        if (node.isFallback()) {
          node.ref.handles.push(hwnd);
        }
        if (node.isLeaf()) {
          const mutated = node.mutateToStacked();
          mutated.ref.handles.push(hwnd);
        }
        return true;
      }
      case Reservation.Left: {
        const mutated = node.mutateToBranch(NodeType.Horizontal);
        mutated.ref.children.unshift(NodeImpl.newLeaf(hwnd, mutated.ref.children[0]!.priority));
        return true;
      }
      case Reservation.Right: {
        const mutated = node.mutateToBranch(NodeType.Horizontal);
        mutated.ref.children.push(NodeImpl.newLeaf(hwnd, mutated.ref.children[0]!.priority));
        return true;
      }
      case Reservation.Top: {
        const mutated = node.mutateToBranch(NodeType.Vertical);
        mutated.ref.children.unshift(NodeImpl.newLeaf(hwnd, mutated.ref.children[0]!.priority));
        return true;
      }
      case Reservation.Bottom: {
        const mutated = node.mutateToBranch(NodeType.Vertical);
        mutated.ref.children.push(NodeImpl.newLeaf(hwnd, mutated.ref.children[0]!.priority));
        return true;
      }
      default:
        console.throw(`Unknown reservation ${reservation}`);
    }
  }

  getNodeContaining(searched: HWND): LeafNode | FallbackNode | null {
    if (this.isLeaf()) {
      return this.ref.handle === searched ? this.ref : null;
    }

    if (this.isFallback()) {
      return this.ref.handles.includes(searched) ? this.ref : null;
    }

    if (this.isBranch()) {
      for (const child of this.ref.children) {
        const result = NodeImpl.from(child).getNodeContaining(searched);
        if (result) {
          return result;
        }
      }
      return null;
    }

    this.unreachable();
  }
}
