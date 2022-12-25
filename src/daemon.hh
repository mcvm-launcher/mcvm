#pragma once
#include "lib/util.hh"
#include "io/files.hh"

#include <uv.h>

#include <thread>

namespace mcvm {
	// Handle to the daemon process
	class Daemon {
		int pid;
		const fs::path pid_path;

		// Run when the daemon exits
		static void on_exit(uv_process_t *req, int64_t exit_status, int term_signal);

		public:
		Daemon(const fs::path& run_dir);

		void start();
		// Start the daemon if it is not started already
		void ensure_started();
		// Init function for the daemon process
		static void daemon_init();
	};
};
