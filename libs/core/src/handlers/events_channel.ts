import type { SelfEventsToken } from "@seelen-ui/types";
import { invoke } from "@tauri-apps/api/core";
import type { EventCallback } from "@tauri-apps/api/event";

import { SeelenCommand } from "./commands.ts";

const RECONNECT_MIN_DELAY_MS = 250;
const RECONNECT_MAX_DELAY_MS = 5_000;
/** Shared across every copy of this lib loaded in the same webview, so there is a single connection. */
const GLOBAL_KEY = Symbol.for("@seelen-ui/lib/events-channel");

type Listener = EventCallback<unknown>;

type ClientMessage = { type: "subscribe"; event: string } | { type: "unsubscribe"; event: string };
type ServerMessage = { type: "ack"; event: string } | { type: "event"; event: string; payload: unknown };

interface Subscription {
  /** resolves once the server acknowledged the subscription (or the connection failed) */
  ready: Promise<void>;
  resolve: () => void;
}

/**
 * Single websocket per webview to receive the global events.
 * Multiple subscriptions to the same event share one server side subscription,
 * so subscribing again only registers the callback locally.
 */
class EventsChannel {
  #socket: WebSocket | null = null;
  #connecting = false;
  #credentials: Promise<SelfEventsToken> | null = null;
  #reconnectDelay = RECONNECT_MIN_DELAY_MS;
  #reconnectTimer: ReturnType<typeof setTimeout> | null = null;

  #nextListenerId = 1;
  /** event -> listener id -> callback */
  #listeners = new Map<string, Map<number, Listener>>();
  /** event -> server side subscription */
  #subscriptions = new Map<string, Subscription>();
  /**
   * event -> resolvers waiting for an ack, in sending order.
   * The server acks in the same order it receives the subscriptions.
   */
  #pendingAcks = new Map<string, Array<() => void>>();

  async subscribe(event: string, cb: Listener): Promise<() => void> {
    let listeners = this.#listeners.get(event);
    if (!listeners) {
      listeners = new Map();
      this.#listeners.set(event, listeners);
    }

    const id = this.#nextListenerId++;
    listeners.set(id, cb);

    let subscription = this.#subscriptions.get(event);
    if (!subscription) {
      subscription = createSubscription();
      this.#subscriptions.set(event, subscription);
      this.#requestSubscription(event, subscription);
      this.#connect();
    }

    await subscription.ready;
    return (): void => this.#unsubscribe(event, id);
  }

  #unsubscribe(event: string, id: number): void {
    const listeners = this.#listeners.get(event);
    if (!listeners?.delete(id) || listeners.size > 0) {
      return;
    }
    this.#listeners.delete(event);
    this.#subscriptions.delete(event);
    this.#send({ type: "unsubscribe", event });
  }

  /** Sends the subscription if connected, otherwise it is sent on (re)connection. */
  #requestSubscription(event: string, subscription: Subscription): void {
    if (this.#send({ type: "subscribe", event })) {
      let queue = this.#pendingAcks.get(event);
      if (!queue) {
        queue = [];
        this.#pendingAcks.set(event, queue);
      }
      queue.push(subscription.resolve);
    }
  }

  #send(message: ClientMessage): boolean {
    if (this.#socket?.readyState !== WebSocket.OPEN) {
      return false;
    }
    this.#socket.send(JSON.stringify(message));
    return true;
  }

  async #connect(): Promise<void> {
    if (this.#socket || this.#connecting) {
      return;
    }
    this.#connecting = true;

    try {
      this.#credentials ??= invoke<SelfEventsToken>(SeelenCommand.GetSelfToken);
      const { token, endpoint } = await this.#credentials;

      const socket = new WebSocket(`${endpoint}?token=${encodeURIComponent(token)}`);
      socket.onopen = (): void => this.#onOpen();
      socket.onmessage = (e: MessageEvent<string>): void => this.#onMessage(e.data);
      socket.onclose = (): void => this.#onClose(socket);
      this.#socket = socket;
    } catch (error) {
      console.error("Failed to connect to the events channel:", error);
      this.#credentials = null;
      this.#onConnectionLost();
    } finally {
      this.#connecting = false;
    }
  }

  #onOpen(): void {
    this.#reconnectDelay = RECONNECT_MIN_DELAY_MS;
    for (const [event, subscription] of this.#subscriptions) {
      this.#requestSubscription(event, subscription);
    }
  }

  #onMessage(data: string): void {
    let message: ServerMessage;
    try {
      message = JSON.parse(data);
    } catch {
      return;
    }

    if (message.type === "ack") {
      this.#pendingAcks.get(message.event)?.shift()?.();
      return;
    }

    const listeners = this.#listeners.get(message.event);
    if (!listeners) {
      return;
    }
    for (const [id, cb] of listeners) {
      try {
        cb({ event: message.event, id, payload: message.payload });
      } catch (error) {
        console.error(`Error on listener of ${message.event}:`, error);
      }
    }
  }

  #onClose(socket: WebSocket): void {
    if (this.#socket === socket) {
      this.#socket = null;
      this.#onConnectionLost();
    }
  }

  #onConnectionLost(): void {
    // acks of the lost connection will never arrive
    this.#pendingAcks.clear();
    // don't block subscribers forever, events will flow once reconnected
    for (const subscription of this.#subscriptions.values()) {
      subscription.resolve();
    }

    if (this.#listeners.size === 0 || this.#reconnectTimer) {
      return;
    }
    this.#reconnectTimer = setTimeout((): void => {
      this.#reconnectTimer = null;
      this.#connect();
    }, this.#reconnectDelay);
    this.#reconnectDelay = Math.min(this.#reconnectDelay * 2, RECONNECT_MAX_DELAY_MS);
  }
}

function createSubscription(): Subscription {
  let resolve!: () => void;
  const ready = new Promise<void>((r) => (resolve = r));
  return { ready, resolve };
}

export function getEventsChannel(): EventsChannel {
  const global = globalThis as unknown as Record<symbol, EventsChannel | undefined>;
  return (global[GLOBAL_KEY] ??= new EventsChannel());
}
