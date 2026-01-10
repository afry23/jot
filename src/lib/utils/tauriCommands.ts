export const testCommand: string = "test_command";

export namespace BackupCommands {
  export const CREATE_BACKUP: string = "create_backup_command";
  export const GET_BACKUP_PATH: string = "get_backup_path_command";
  export const RESTORE_FROM_BACKUP: string = "restore_from_backup_command";
  export const GET_BACKUP_LIST: string = "get_backup_list_command";
  export const GET_BACKUP_DETAILS: string = "get_backup_details_command";
  export const DELETE_BACKUP: string = "delete_backup_command";
}

export namespace SettingsCommands {
  export const SAVE_SETTINGS: string = "save_settings";
  export const LOAD_SETTINGS: string = "load_settings";
  export const SAVE_ACTIVE_TAB: string = "save_active_tab";
  export const HAS_LANGUAGETOOL_API_KEY: string = "has_languagetool_credential";
  export const HAS_DEEPL_API_KEY: string = "has_deepl_credential";
}

export namespace NotesCommands {
  export const LOAD_NOTES: string = "load_notes";
  export const SAVE_NOTES: string = "save_notes";
}

export namespace UlitilyCommands {
  export const HIDE_WINDOW: string = "hide_window";
}
