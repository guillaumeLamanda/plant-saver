---@module "lazy"
---@type LazySpec
return {
	{
		"mrcjkb/rustaceanvim",
		opts = function(_, opts)
			opts = opts or {}
			opts.server = opts.server or {}
			opts.server.default_settings = vim.tbl_deep_extend("force", opts.server.default_settings or {}, {
				["rust-analyzer"] = {
					cargo = {
						target = "riscv32imac-esp-espidf",
						allFeatures = true,
						buildScripts = {
							enable = true,
						},
						loadOutDirsFromCheck = true,
					},
					procMacro = {
						enable = true,
					},
					diagnostics = {
						disabled = { "inactive-code", "unresolved-proc-macro" },
					},
				},
			})
		end,
	},
	{
		"jmbuhr/otter.nvim",
		dependencies = {
			"nvim-treesitter/nvim-treesitter",
		},
		config = function()
			vim.api.nvim_create_autocmd({ "FileType" }, {
				pattern = { "toml" },
				group = vim.api.nvim_create_augroup("EmbedToml", {}),
				callback = function()
					require("otter").activate()
				end,
			})
		end,
	},
}
