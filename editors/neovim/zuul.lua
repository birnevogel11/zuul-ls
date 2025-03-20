local Zuul = {
  exec = os.getenv("ZUUL_SEARCH_BIN_PATH") or "zuul-search",
}

Zuul.get_visual_selection = function()
  vim.cmd('noau normal! "vy"')
  local text = vim.fn.getreg("v")
  vim.fn.setreg("v", {})

  ---@diagnostic disable-next-line: param-type-mismatch
  text = string.gsub(text, "\n", "")
  if #text > 0 then
    return text
  else
    return ""
  end
end

---@param input string
---@return table<string>
local split_by_tab = function(input)
  local result = {}
  for word in string.gmatch(input, "[^\t]+") do
    table.insert(result, word)
  end
  return result
end

---@param cmd string
---@return table<string>
local function get_command_output(cmd)
  local lines = {}

  local handle = io.popen(cmd)
  if handle then
    for line in handle:lines() do
      table.insert(lines, line)
    end
    handle:close()
  else
    print("Failed to execute fd command")
  end

  return lines
end

---@param cmd string
---@return table<table<string>>?
local function get_command_output_spilt_by_tab(cmd)
  local lines = get_command_output(cmd)
  if not lines then
    return nil
  end

  local token_lines = {}
  for _, job in ipairs(lines) do
    local tokens = split_by_tab(job)
    token_lines[#token_lines+1] = tokens
  end

  return token_lines
end

---@param sub_command string
---@param job_name string?
Zuul._make_command = function(sub_command, job_name)
  local cmd = Zuul.exec .. " " .. sub_command
  if job_name ~= nil then
    cmd = cmd .. " " .. job_name
  end
  return cmd
end


---@param sub_command string
---@param transfer fun(tokens:table<string>):snacks.picker.Item
Zuul._transfer_items = function(sub_command, transfer)
  local token_lines = get_command_output_spilt_by_tab(Zuul._make_command(sub_command))
  if not token_lines then
    return
  end

  local items = {}
  for _, tokens in ipairs(token_lines) do
    table.insert(items, transfer(tokens))
  end

  return items
end

---@param base_opts snacks.picker.Config
---@param opts snacks.picker.Config
---@return snacks.Picker
Zuul._picker = function(base_opts, opts)
  return Snacks.picker(vim.tbl_deep_extend("force", base_opts, opts))
end

---@param tokens table<string>
---@return snacks.picker.Item
local function transfer_file_loc(tokens)
    return {
      text = tokens[1],
      file = tokens[2],
      pos = {tonumber(tokens[3]) + 1, tonumber(tokens[4])},
    }
end

---@param tokens table<string>
---@return snacks.picker.Item
local function transfer_file(tokens)
    return {
      text = tokens[1],
      file = tokens[2],
    }
end

---@param tokens table<string>
---@return snacks.picker.Item
local function transfer_var(tokens)
    return {
      text = tokens[1],
      file = tokens[4],
      pos = {(tonumber(tokens[5]) or 0) + 1, tonumber(tokens[6])},
      zuul_job = tokens[2],
      zuul_value = tokens[3],
    }
end

---@param item snacks.picker.Item
---@return snacks.picker.Highlight
local function format_text_only(item)
  return {{item.text, "SnacksPickerItem"}}
end

---@param opts snacks.picker.Config
---@return snacks.Picker
Zuul.job = function(opts)
  local base_opts = {
    title = "Zuul Jobs",
    items = Zuul._transfer_items("jobs", transfer_file),
    format = format_text_only,
  }

  return Zuul._picker(base_opts, opts)
end

---@param opts snacks.picker.Config
---@return snacks.Picker
Zuul.project_template = function(opts)
  local base_opts = {
    title = "Zuul Project Templates",
    items = Zuul._transfer_items("project-templates", transfer_file_loc),
    format = format_text_only,
  }

  return Zuul._picker(base_opts, opts)
end

---@param opts snacks.picker.Config
---@return snacks.Picker
Zuul.workspace_var = function(opts)
  local a = Snacks.picker.util.align

  local base_opts = {
    title = "Zuul Workspace Vars",
    items = Zuul._transfer_items("workdir-vars", transfer_var),
    format = function(item)
      return {
        {a(item.text, 40), "SnacksPickerItem"},
        {" ", "SnacksComment"},
        {a(item.zuul_job, 40), "SnacksComment"},
        {" ", "SnacksComment"},
        {a(item.zuul_value, 50), "SnacksComment"},
      }
    end,
  }

  return Zuul._picker(base_opts, opts)
end

---@param opts snacks.picker.Config
---@return snacks.Picker
Zuul.role = function(opts)
  local a = Snacks.picker.util.align

  local base_opts = {
    title = "Zuul Roles",
    items = Zuul._transfer_items("roles", transfer_file),
    format = function(item)
      return {
        {a(item.text, 50), "SnacksPickerItem"},
        {" ", "SnacksComment"},
        {item.file, "SnacksPickerFile"},
      }
    end,
  }

  return Zuul._picker(base_opts, opts)
end


---@param opts snacks.picker.Config
---@param job_name string?
---@return snacks.Picker
Zuul.job_hierarchy = function(opts, job_name)
  job_name = job_name or Zuul.get_visual_selection()

  local base_opts = {
    title = "Zuul Job Hierarchy",
    items = Zuul._transfer_items("job-hierarchy " .. job_name, transfer_file_loc),
    format = format_text_only,
  }

  return Zuul._picker(base_opts, opts)
end


---@param opts snacks.picker.Config
---@param job_name string?
---@return snacks.Picker
Zuul.job_playbooks =  function(opts, job_name)
  job_name = job_name or Zuul.get_visual_selection()

  local base_opts = {
    title = "Zuul Job Playbooks",
    items = Zuul._transfer_items("job-playbooks " .. job_name, transfer_file),
    format = format_text_only,
  }

  return Zuul._picker(base_opts, opts)
end

return Zuul
