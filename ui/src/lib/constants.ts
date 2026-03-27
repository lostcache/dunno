import type { SchemaField, EdgePair, NodeColor } from './types'

export const ENTITY_TABS = [
  'projects', 'modules', 'files', 'tasks', 'todos',
  'user-stories', 'epics', 'personas', 'workflows', 'contexts', 'issues',
]

export const SCHEMAS: Record<string, SchemaField[]> = {
  projects: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea', required: true },
  ],
  modules: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea', required: true },
    { name: 'notes', label: 'Notes', type: 'textarea' },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
    { name: 'parent_module_id', label: 'Parent Module ID (optional)', type: 'text' },
  ],
  files: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'path', label: 'Path', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'notes', label: 'Notes', type: 'textarea' },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
    { name: 'parent_id', label: 'Parent Module ID (optional)', type: 'text' },
  ],
  tasks: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea', required: true },
    { name: 'module_id', label: 'Module ID', type: 'text' },
    { name: 'project_id', label: 'Project ID', type: 'text', fill: 'projectId' },
  ],
  todos: [
    { name: 'content', label: 'Content', type: 'textarea', required: true },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
  ],
  'user-stories': [
    { name: 'title', label: 'Title', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea', required: true },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
  ],
  epics: [
    { name: 'title', label: 'Title', type: 'text', required: true },
    { name: 'description', label: 'Description', type: 'textarea', required: true },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
  ],
  personas: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'content', label: 'Content', type: 'textarea', required: true },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
  ],
  workflows: [
    { name: 'name', label: 'Name', type: 'text', required: true },
    { name: 'content', label: 'Content', type: 'textarea', required: true },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
  ],
  contexts: [
    { name: 'link_to', label: 'Link To (node ID)', type: 'text', required: true },
    { name: 'fields_type', label: 'Type', type: 'text', required: true },
    { name: 'fields_content', label: 'Content', type: 'textarea' },
    { name: 'fields_description', label: 'Description', type: 'textarea' },
  ],
  issues: [
    { name: 'description', label: 'Description', type: 'textarea', required: true },
    { name: 'plan', label: 'Plan', type: 'textarea' },
    { name: 'project_id', label: 'Project ID', type: 'text', required: true, fill: 'projectId' },
    { name: 'task_id', label: 'Task (optional)', type: 'select', fill: 'taskId' },
  ],
}

export const EDIT_SCHEMAS: Record<string, SchemaField[]> = {
  project: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
  ],
  module: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'notes', label: 'Notes', type: 'textarea' },
  ],
  file: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'path', label: 'Path', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'notes', label: 'Notes', type: 'textarea' },
  ],
  task: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'status', label: 'Status', type: 'select', options: ['pending', 'active', 'completed'] },
  ],
  todo_item: [
    { name: 'content', label: 'Content', type: 'textarea' },
  ],
  user_story: [
    { name: 'title', label: 'Title', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
  ],
  epic: [
    { name: 'title', label: 'Title', type: 'text' },
    { name: 'description', label: 'Description', type: 'textarea' },
  ],
  persona: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'content', label: 'Content', type: 'textarea' },
  ],
  workflow: [
    { name: 'name', label: 'Name', type: 'text' },
    { name: 'content', label: 'Content', type: 'textarea' },
  ],
  context: [
    { name: 'type', label: 'Type', type: 'text' },
    { name: 'content', label: 'Content', type: 'textarea' },
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'example', label: 'Example', type: 'textarea' },
    { name: 'severity', label: 'Severity', type: 'text' },
    { name: 'category', label: 'Category', type: 'text' },
  ],
  issue: [
    { name: 'description', label: 'Description', type: 'textarea' },
    { name: 'plan', label: 'Plan', type: 'textarea' },
    { name: 'status', label: 'Status', type: 'select', options: ['pending', 'active', 'completed'] },
  ],
}

export const EDIT_ENDPOINTS: Record<string, string> = {
  project: '/api/projects',
  module: '/api/modules',
  file: '/api/files',
  task: '/api/tasks',
  todo_item: '/api/todos',
  user_story: '/api/user-stories',
  epic: '/api/epics',
  persona: '/api/personas',
  workflow: '/api/workflows',
  context: '/api/contexts',
  issue: '/api/issues',
}

export const EDGE_PAIRS: EdgePair[] = [
  { a: 'project',    b: 'module',     a_to_b: 'contains',       b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'file',       a_to_b: 'contains',       b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'task',       a_to_b: 'has_task',       b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'user_story', a_to_b: 'has_user_story', b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'epic',       a_to_b: 'has_epic',       b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'todo_item',  a_to_b: 'has_todo',       b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'persona',    a_to_b: 'has_persona',    b_to_a: 'belongs_to_project' },
  { a: 'project',    b: 'workflow',   a_to_b: 'has_workflow',   b_to_a: 'belongs_to_project' },
  { a: 'module',     b: 'module',     a_to_b: 'has_module',     b_to_a: 'belongs_to_module' },
  { a: 'module',     b: 'file',       a_to_b: 'contains',       b_to_a: 'belongs_to_module' },
  { a: 'module',     b: 'task',       a_to_b: 'has_task',       b_to_a: 'belongs_to_module' },
  { a: 'epic',       b: 'user_story', a_to_b: 'has_user_story', b_to_a: 'belongs_to_epic' },
  { a: 'epic',       b: 'task',       a_to_b: 'has_task',       b_to_a: 'belongs_to_epic' },
  { a: 'user_story', b: 'task',       a_to_b: 'has_task',       b_to_a: 'belongs_to_story' },
]

export const ALL_EDGE_TYPES = [
  'contains', 'has_module', 'has_task', 'has_context', 'belongs_to_project', 'belongs_to_module',
  'belongs_to_task', 'belongs_to_story', 'has_todo',
  'has_user_story', 'belongs_to_user_story', 'belongs_to_epic', 'has_epic',
  'has_persona', 'has_workflow', 'has_issue',
]

export const CREATE_ENDPOINTS: Record<string, string> = {
  projects: '/api/projects',
  modules: '/api/modules',
  files: '/api/files',
  tasks: '/api/tasks',
  todos: '/api/todos',
  'user-stories': '/api/user-stories',
  epics: '/api/epics',
  personas: '/api/personas',
  workflows: '/api/workflows',
  contexts: '/api/contexts',
  issues: '/api/issues',
}

export const TYPE_MAP: Record<string, string> = {
  projects: 'project',
  modules: 'module',
  files: 'file',
  tasks: 'task',
  todos: 'todo_item',
  'user-stories': 'user_story',
  epics: 'epic',
  personas: 'persona',
  workflows: 'workflow',
  contexts: 'context',
  issues: 'issue',
}

export const NODE_COLORS: Record<string, NodeColor> = {
  project:    { bg: '#3b82f6', fg: '#fff' },
  module:     { bg: '#8b5cf6', fg: '#fff' },
  file:       { bg: '#22c55e', fg: '#fff' },
  task:       { bg: '#f97316', fg: '#fff' },
  context:    { bg: '#ef4444', fg: '#fff' },
  epic:       { bg: '#eab308', fg: '#000' },
  user_story: { bg: '#06b6d4', fg: '#fff' },
  todo_item:  { bg: '#64748b', fg: '#fff' },
  persona:    { bg: '#ec4899', fg: '#fff' },
  workflow:   { bg: '#14b8a6', fg: '#fff' },
  issue:      { bg: '#f43f5e', fg: '#fff' },
}

export const FRIENDLY_TYPES: Record<string, string> = {
  project:    'Project',
  module:     'Module',
  file:       'File',
  task:       'Task',
  context:    'Knowledge',
  epic:       'Epic',
  user_story: 'User Story',
  todo_item:  'Todo',
  persona:    'Persona',
  workflow:   'Workflow',
  issue:      'Issue',
}

export function findEdgePair(typeA: string, typeB: string): EdgePair | undefined {
  return EDGE_PAIRS.find(p =>
    (p.a === typeA && p.b === typeB) || (p.a === typeB && p.b === typeA)
  )
}
