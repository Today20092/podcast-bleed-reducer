using System;
using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;

namespace CleanSlate.Build {
    public sealed class MemorySample {
        public ulong AvailablePhysical, AvailableCommit;
    }
    public sealed class TaskSample {
        public uint ActiveProcesses;
        public ulong PrivateBytes, WorkingSetBytes, PeakCommitBytes;
        public bool MemoryLimitHit;
    }
    public sealed class OwnedTask : IDisposable {
        IntPtr job, process, port;
        bool memoryLimitHit;
        public uint ProcessId { get; private set; }
        public string JobName { get; private set; }
        [StructLayout(LayoutKind.Sequential)] struct MemoryStatus {
            public uint Length, Load;
            public ulong TotalPhysical, AvailablePhysical, TotalPageFile, AvailablePageFile, TotalVirtual, AvailableVirtual, AvailableExtendedVirtual;
        }
        [StructLayout(LayoutKind.Sequential)] struct BasicLimits {
            public long ProcessTime, JobTime;
            public uint Flags;
            public UIntPtr MinWorkingSet, MaxWorkingSet;
            public uint ActiveProcessLimit;
            public UIntPtr Affinity;
            public uint Priority, Scheduling;
        }
        [StructLayout(LayoutKind.Sequential)] struct IoCounters {
            public ulong ReadOps, WriteOps, OtherOps, ReadBytes, WriteBytes, OtherBytes;
        }
        [StructLayout(LayoutKind.Sequential)] struct ExtendedLimits {
            public BasicLimits Basic;
            public IoCounters Io;
            public UIntPtr ProcessMemory, JobMemory, PeakProcessMemory, PeakJobMemory;
        }
        [StructLayout(LayoutKind.Sequential)] struct Accounting {
            public long User, Kernel, PeriodUser, PeriodKernel;
            public uint Faults, TotalProcesses, ActiveProcesses, TerminatedProcesses;
        }
        [StructLayout(LayoutKind.Sequential)] struct CompletionPort { public IntPtr Key, Port; }
        [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] struct StartupInfo {
            public int Size;
            public string Reserved, Desktop, Title;
            public uint X, Y, Width, Height, XChars, YChars, Fill, Flags;
            public ushort Show, ReservedSize;
            public IntPtr ReservedPtr, Input, Output, Error;
        }
        [StructLayout(LayoutKind.Sequential)] struct ProcessInfo { public IntPtr Process, Thread; public uint Id, ThreadId; }
        [StructLayout(LayoutKind.Sequential)] struct SecurityAttributes { public int Size; public IntPtr Descriptor; public int Inherit; }
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool GlobalMemoryStatusEx(ref MemoryStatus status);
        [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr CreateJobObject(IntPtr attributes, string name);
        [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr OpenJobObject(uint access, bool inherit, string name);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool IsProcessInJob(IntPtr process, IntPtr job, out bool contained);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool SetInformationJobObject(IntPtr job, int kind, IntPtr data, uint size);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool QueryInformationJobObject(IntPtr job, int kind, IntPtr data, uint size, IntPtr returned);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool AssignProcessToJobObject(IntPtr job, IntPtr process);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool TerminateJobObject(IntPtr job, uint code);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool TerminateProcess(IntPtr process, uint code);
        [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern bool CreateProcess(string application, StringBuilder command, IntPtr processAttributes, IntPtr threadAttributes, bool inherit, uint flags, IntPtr environment, string directory, ref StartupInfo startup, out ProcessInfo process);
        [DllImport("kernel32.dll", SetLastError=true)] static extern uint ResumeThread(IntPtr thread);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool CloseHandle(IntPtr handle);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool GetExitCodeProcess(IntPtr process, out uint code);
        [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr CreateIoCompletionPort(IntPtr file, IntPtr existing, UIntPtr key, uint threads);
        [DllImport("kernel32.dll", SetLastError=true)] static extern bool GetQueuedCompletionStatus(IntPtr port, out uint message, out UIntPtr key, out IntPtr value, uint timeout);
        [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr CreateFile(string path, uint access, uint share, ref SecurityAttributes attributes, uint creation, uint flags, IntPtr template);
        static void Check(bool success) { if (!success) throw new Win32Exception(Marshal.GetLastWin32Error()); }
        static IntPtr Structure<T>(T value) {
            IntPtr p = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(T)));
            Marshal.StructureToPtr(value, p, false); return p;
        }
        T Query<T>(int kind) {
            int size = Marshal.SizeOf(typeof(T)); IntPtr p = Marshal.AllocHGlobal(size);
            try { Check(QueryInformationJobObject(job, kind, p, (uint)size, IntPtr.Zero)); return (T)Marshal.PtrToStructure(p, typeof(T)); }
            finally { Marshal.FreeHGlobal(p); }
        }
        void Set<T>(int kind, T value) {
            IntPtr p = Structure(value);
            try { Check(SetInformationJobObject(job, kind, p, (uint)Marshal.SizeOf(typeof(T)))); }
            finally { Marshal.FreeHGlobal(p); }
        }
        public static MemorySample Memory() {
            var m = new MemoryStatus(); m.Length = (uint)Marshal.SizeOf(typeof(MemoryStatus)); Check(GlobalMemoryStatusEx(ref m));
            return new MemorySample { AvailablePhysical=m.AvailablePhysical, AvailableCommit=m.AvailablePageFile };
        }
        public static bool IsContained(string name) {
            if (String.IsNullOrEmpty(name)) return false;
            IntPtr j = OpenJobObject(4, false, name);
            if (j == IntPtr.Zero) return false;
            try { bool contained; Check(IsProcessInJob(Process.GetCurrentProcess().Handle, j, out contained)); return contained; }
            finally { CloseHandle(j); }
        }
        // Windows command-line quoting; executable and arguments never pass through a shell parser.
        static string Quote(string value) {
            var b = new StringBuilder("\""); int slashes=0;
            foreach (char c in value) {
                if (c == '\\') { slashes++; continue; }
                b.Append('\\', c == '"' ? slashes * 2 + 1 : slashes); b.Append(c); slashes=0;
            }
            b.Append('\\', slashes * 2); return b.Append('"').ToString();
        }
        public OwnedTask(string executable, string[] arguments, string directory, ulong commitLimit, string logPath) {
            IntPtr output=IntPtr.Zero, input=IntPtr.Zero, thread=IntPtr.Zero;
            try {
                JobName = "Local\\CleanSlate.OwnedTask." + Guid.NewGuid().ToString("N");
                job=CreateJobObject(IntPtr.Zero, JobName); Check(job != IntPtr.Zero);
                Set(9, new ExtendedLimits { Basic=new BasicLimits { Flags=0x2000 | 0x200 }, JobMemory=new UIntPtr(commitLimit) });
                port=CreateIoCompletionPort(new IntPtr(-1), IntPtr.Zero, UIntPtr.Zero, 1); Check(port != IntPtr.Zero);
                Set(7, new CompletionPort { Key=new IntPtr(1), Port=port });
                var attributes=new SecurityAttributes { Size=Marshal.SizeOf(typeof(SecurityAttributes)), Inherit=1 };
                output=CreateFile(logPath, 0x40000000, 3, ref attributes, 1, 0x80, IntPtr.Zero); Check(output != new IntPtr(-1));
                input=CreateFile("NUL", 0x80000000, 3, ref attributes, 3, 0x80, IntPtr.Zero); Check(input != new IntPtr(-1));
                var startup=new StartupInfo { Size=Marshal.SizeOf(typeof(StartupInfo)), Flags=0x100, Input=input, Output=output, Error=output };
                var command=new StringBuilder(Quote(executable)); foreach (string a in arguments) command.Append(" ").Append(Quote(a));
                ProcessInfo info;
                string previous=Environment.GetEnvironmentVariable("CLEANSLATE_RESOURCE_JOB");
                try {
                    Environment.SetEnvironmentVariable("CLEANSLATE_RESOURCE_JOB", JobName);
                    Check(CreateProcess(executable, command, IntPtr.Zero, IntPtr.Zero, true, 0x08000004, IntPtr.Zero, directory, ref startup, out info));
                } finally { Environment.SetEnvironmentVariable("CLEANSLATE_RESOURCE_JOB", previous); }
                process=info.Process; thread=info.Thread; ProcessId=info.Id;
                try { Check(AssignProcessToJobObject(job, process)); Check(ResumeThread(thread) != UInt32.MaxValue); }
                catch { TerminateProcess(process, 125); throw; }
            } catch { Dispose(); throw; }
            finally {
                if (thread != IntPtr.Zero) CloseHandle(thread);
                if (output != IntPtr.Zero && output != new IntPtr(-1)) CloseHandle(output);
                if (input != IntPtr.Zero && input != new IntPtr(-1)) CloseHandle(input);
            }
        }
        public TaskSample Sample() {
            uint message; UIntPtr key; IntPtr value;
            while (GetQueuedCompletionStatus(port, out message, out key, out value, 0)) if (message == 9 || message == 10) memoryLimitHit=true;
            var sample=new TaskSample { ActiveProcesses=Query<Accounting>(1).ActiveProcesses, PeakCommitBytes=Query<ExtendedLimits>(9).PeakJobMemory.ToUInt64(), MemoryLimitHit=memoryLimitHit };
            int capacity=4096, size=8+capacity*IntPtr.Size; IntPtr ids=Marshal.AllocHGlobal(size);
            try {
                Check(QueryInformationJobObject(job, 3, ids, (uint)size, IntPtr.Zero));
                int count=Marshal.ReadInt32(ids,4);
                if (count > capacity) throw new InvalidOperationException("Owned process count exceeds accounting capacity.");
                for (int i=0;i<count;i++) {
                    int id=checked((int)Marshal.ReadIntPtr(ids,8+i*IntPtr.Size).ToInt64());
                    try { using (Process p=Process.GetProcessById(id)) { sample.PrivateBytes+=(ulong)p.PrivateMemorySize64; sample.WorkingSetBytes+=(ulong)p.WorkingSet64; } }
                    catch (ArgumentException) { } // The owned child exited between job accounting and the process read.
                    catch (InvalidOperationException) { }
                }
            } finally { Marshal.FreeHGlobal(ids); }
            return sample;
        }
        public uint ExitCode { get { uint code; Check(GetExitCodeProcess(process, out code)); return code; } }
        public bool Stop() {
            Check(TerminateJobObject(job, 125));
            var timer=Stopwatch.StartNew();
            while (Query<Accounting>(1).ActiveProcesses != 0 && timer.ElapsedMilliseconds < 5000) System.Threading.Thread.Sleep(20);
            return Query<Accounting>(1).ActiveProcesses == 0;
        }
        public void Dispose() {
            if (job != IntPtr.Zero) { CloseHandle(job); job=IntPtr.Zero; }
            if (process != IntPtr.Zero) { CloseHandle(process); process=IntPtr.Zero; }
            if (port != IntPtr.Zero) { CloseHandle(port); port=IntPtr.Zero; }
        }
    }
}
