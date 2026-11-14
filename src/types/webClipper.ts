export interface AuthorizedExtensionInfo {
  extension_id: string;
  browser?: string;
  paired_at: string;
}

export interface ClipperPairingInfo {
  port: number;
  token_configured: boolean;
  is_running: boolean;
  authorized_extension_id?: string;
  authorized_extensions: AuthorizedExtensionInfo[];
}

export interface ClipperConnectionTestResult {
  reachable: boolean;
  port: number;
  is_running: boolean;
  message: string;
}
