/**
 * Local Storage Service Contracts
 *
 * Defines interface boundaries for accessing local metadata and configuration.
 */

export interface StorageServiceContract {
  getItem<T>(key: string): Promise<T | null>;
  setItem<T>(key: string, value: T): Promise<void>;
  removeItem(key: string): Promise<void>;
}

