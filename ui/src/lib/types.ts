export interface SchemaField {
  name: string;
  label: string;
  type: "text" | "textarea" | "select";
  required?: boolean;
  fill?: "projectId" | "taskId";
  options?: string[];
}

export interface EdgePair {
  a: string;
  b: string;
  a_to_b: string;
  b_to_a: string;
}

export interface NodeColor {
  bg: string;
  fg: string;
}

export interface NodeData {
  id: string;
  label: string;
  node_type: string;
  [key: string]: unknown;
}
