import os
import time


def run_cmd(cmd):
    print('Running command: {}'.format(cmd))
    os.system(cmd)


loop_no = 0
while True:
    time.sleep(5)
    print(f'Checking for updates ({loop_no})...')
    loop_no += 1
    for f in os.listdir('.'):
        if f.startswith('update_'):
            run_cmd('chmod +x {}'.format(f))
            run_cmd('systemctl stop web-portal')
            run_cmd('rm -rf ../bin/web-portal')
            run_cmd('mv {} ../bin/web-portal'.format(f))
            run_cmd('systemctl start web-portal')
