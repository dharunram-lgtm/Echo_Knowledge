import { invoke } from "@tauri-apps/api/core";
import type {
  ClipperPairingInfo,
  ClipperConnectionTestResult,
} from "@/types/webClipper";

export class WebClipperService {
  /**
   * Retrieves current server status, active port, whether token is configured,
   * and authorized browser extensions WITHOUT returning the sensitive plaintext token.
   */
  static async getPairingInfo(): Promise<ClipperPairingInfo> {
    return invoke<ClipperPairingInfo>("get_clipper_pairing_info");
  }

  /**
   * Explicitly requests the plaintext pairing token from Windows Credential Manager.
   * Triggered only when the user explicitly reveals or copies the token.
   */
  static async revealToken(): Promise<string> {
    return invoke<string>("reveal_clipper_pairing_token");
  }

  /**
   * Starts or stops the loopback HTTP server.
   */
  static async setServerState(enabled: boolean): Promise<boolean> {
    return invoke<boolean>("set_clipper_server_state", { enabled });
  }

  /**
   * Generates a new 256-bit pairing token, invalidating all previously paired browser extensions.
   */
  static async regenerateToken(): Promise<string> {
    return invoke<string>("regenerate_clipper_pairing_token");
  }

  /**
   * Revokes the pairing token and deauthorizes all browser extensions.
   */
  static async revokeToken(): Promise<boolean> {
    return invoke<boolean>("revoke_clipper_pairing_token");
  }

  /**
   * Authorizes a browser extension ID.
   */
  static async authorizeExtension(
    extensionId: string,
    browser?: string
  ): Promise<boolean> {
    return invoke<boolean>("authorize_clipper_extension", {
      extensionId,
      browser: browser || "Chromium Browser",
    });
  }

  /**
   * Deauthorizes an individual extension ID, or "*" to deauthorize all.
   */
  static async deauthorizeExtension(extensionId: string): Promise<boolean> {
    return invoke<boolean>("deauthorize_clipper_extension", { extensionId });
  }

  /**
   * Tests loopback server connectivity via backend IPC socket connection.
   */
  static async testConnection(): Promise<ClipperConnectionTestResult> {
    return invoke<ClipperConnectionTestResult>("test_clipper_server_connection");
  }
}
