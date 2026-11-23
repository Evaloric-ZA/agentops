// A small helper for loading data in a component.
//
// The pattern is the same everywhere: a resource has a status (idle,
// loading, loaded, empty, failed), the data, and the error. Components
// render the state rather than juggling booleans themselves.

import { writable, type Readable } from "svelte/store";

export type LoadStatus = "idle" | "loading" | "loaded" | "empty" | "failed";

export interface Resource<T> {
  status: LoadStatus;
  data: T | null;
  error: string | null;
}

/** A resource store: readable, and loadable on demand. */
export interface ResourceStore<T> extends Readable<Resource<T>> {
  load: () => Promise<void>;
}

/** Create a resource store plus a `load` function. */
export function createResource<T>(
  loader: () => Promise<T>,
  isEmpty: (value: T) => boolean = () => false,
): ResourceStore<T> {
  const resource = writable<Resource<T>>({
    status: "idle",
    data: null,
    error: null,
  });

  async function load(): Promise<void> {
    resource.set({ status: "loading", data: null, error: null });
    try {
      const value = await loader();
      resource.set({
        status: isEmpty(value) ? "empty" : "loaded",
        data: value,
        error: null,
      });
    } catch (error) {
      resource.set({
        status: "failed",
        data: null,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  }

  return { subscribe: resource.subscribe, load };
}
