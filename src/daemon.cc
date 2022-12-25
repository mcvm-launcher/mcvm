#include "daemon.hh"

uv_loop_t* loop;
uv_process_t child_req;
uv_process_options_t options;

namespace mcvm {
	Daemon::Daemon(const fs::path& run_dir)
	: pid_path(run_dir / "mcvm.pid") {}

	void Daemon::on_exit(uv_process_t *req, int64_t exit_status, int term_signal) {
		uv_close((uv_handle_t*) req, NULL);
	}

	void Daemon::start() {
		OUT("Starting daemon...");

		loop = uv_default_loop();

		// Args
		char* args[3];
		args[0] = "mcvm";
		args[1] = "__daemon_start__";
		args[2] = NULL;

		options.exit_cb = &on_exit;
		options.args = args;
		options.file = "mcvm";
		options.flags = UV_PROCESS_DETACHED;

		int r = uv_spawn(loop, &child_req, &options);
		if (r) {
			ERR(uv_strerror(r));
			return;
		}
		
		// Write to a file so we can check later
		write_file(pid_path, std::to_string(child_req.pid).c_str());

		const int daemon_exit_code = uv_run(loop, UV_RUN_DEFAULT);
	}

	void Daemon::ensure_started() {
		std::string pid_contents;
		if (file_exists(pid_path)) {
			read_file(pid_path, pid_contents);
		} else {
			start();
			return;
		}

		pid = std::stoi(pid_contents);

		LOG(pid);

		// Check if it exists
		// TODO: Windows support
		#ifdef __linux__
			if (!file_exists("/proc/" + pid)) {
				start();
			}
		#endif
	}

	void Daemon::daemon_init() {
		system("mkdir /home/pango/test/test2");
		while (true) {
			// std::this_thread::yield();
		}
	}
};
