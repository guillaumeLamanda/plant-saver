---@module "lazy"
---@type LazySpec
return {
	{
		"mrcjkb/rustaceanvim",
		opts = function(_, opts)
			opts = opts or {}
			opts.server = opts.server or {}
			-- opts.server.cmd = {
			-- 	"/Users/guillaume.lamanda/.local/share/nvim/rust-analyzer-bin/rust-analyzer-2026-05-04",
			-- }

			-- opts.server.cmd_env = vim.tbl_extend("force", opts.server.cmd_env or {}, {
			-- RUSTUP_TOOLCHAIN = "esp",
			-- LIBCLANG_PATH = "/Users/guillaume.lamanda/.rustup/toolchains/esp/xtensa-esp32-elf-clang/esp-20.1.1_20250829/esp-clang/lib",
			-- PATH = "/Users/guillaume.lamanda/.rustup/toolchains/esp/xtensa-esp-elf/esp-15.2.0_20250920/xtensa-esp-elf/bin:/Users/guillaume.lamanda/.rustup/toolchains/esp/bin:/usr/local/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
			-- })

			opts.server.default_settings = vim.tbl_deep_extend("force", opts.server.default_settings or {}, {
				["rust-analyzer"] = {
					cargo = {
						target = "xtensa-esp32-espidf",
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
