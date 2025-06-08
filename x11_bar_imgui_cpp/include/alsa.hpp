#ifndef ALSA_H
#define ALSA_H

#include <alsa/asoundlib.h>
#include <cmath>
#include <cstdio>
#include <utility>

class Alsa
{
  public:
    // use command `amixer scontrols` to find out `master_mixer` value if that changes
    explicit Alsa(const char* card = "default", const char* master_mixer = "Master")
    {
        if (snd_mixer_open(&m_mixer, 0) < 0)
        {
            perror("alsa: failed to open mixer");
            return;
        }
        if (snd_mixer_attach(m_mixer, card) < 0)
        {
            perror("alsa: failed to attach mixer");
            return;
        }
        if (snd_mixer_selem_register(m_mixer, nullptr, nullptr) < 0)
        {
            perror("alsa: failed to register mixer selem");
            return;
        }
        if (snd_mixer_load(m_mixer) < 0)
        {
            perror("alsa: failed to load mixer");
            return;
        }

        snd_mixer_selem_id_alloca(&m_sid);
        snd_mixer_selem_id_set_index(m_sid, 0);
        snd_mixer_selem_id_set_name(m_sid, master_mixer);
        m_elem = snd_mixer_find_selem(m_mixer, m_sid);
        if (!m_elem)
        {
            perror("alsa: failed to find selem");
            return;
        }

        if (snd_mixer_selem_get_playback_volume_range(m_elem, &m_min_vol, &m_max_vol) < 0)
        {
            perror("alsa: error getting playback volume range");
            m_min_vol = -1;
            m_max_vol = -1;
            return;
        }

        if (snd_mixer_selem_get_playback_volume(m_elem, SND_MIXER_SCHN_FRONT_LEFT, &m_current_vol) < 0)
        {
            perror("alsa: error getting playback volume");
            m_current_vol = -1;
            return;
        }

        if (snd_mixer_selem_has_playback_switch(m_elem))
        {
            int value;
            if (snd_mixer_selem_get_playback_switch(m_elem, SND_MIXER_SCHN_FRONT_LEFT, &value) < 0)
            {
                perror("alsa: failed to get playback switch value");
            }
            else
            {
                m_muted = (bool)value;
            }
        }
        m_muted = m_current_vol == m_min_vol;
    }

    ~Alsa()
    {
        if (m_mixer)
        {
            snd_mixer_close(m_mixer);
            m_mixer = nullptr;
        }
    }

    Alsa(Alsa&)            = delete;
    Alsa& operator=(Alsa&) = delete;

    Alsa(Alsa&& other)
        : m_mixer(std::exchange(other.m_mixer, nullptr)), m_sid(std::exchange(other.m_sid, nullptr)),
          m_elem(std::exchange(other.m_elem, nullptr)), m_min_vol(std::exchange(other.m_min_vol, -1)),
          m_max_vol(std::exchange(other.m_max_vol, -1)), m_current_vol(std::exchange(other.m_current_vol, -1))
    {
    }
    Alsa& operator=(Alsa&& other)
    {
        if (this != &other)
        {
            m_mixer       = std::exchange(other.m_mixer, nullptr);
            m_sid         = std::exchange(other.m_sid, nullptr);
            m_elem        = std::exchange(other.m_elem, nullptr);
            m_min_vol     = std::exchange(other.m_min_vol, -1);
            m_max_vol     = std::exchange(other.m_max_vol, -1);
            m_current_vol = std::exchange(other.m_current_vol, -1);
        }
        return *this;
    }

    void refresh_current_volume()
    {
        if (m_min_vol != -1 && m_max_vol != -1)
        {
            if (snd_mixer_selem_get_playback_volume(m_elem, SND_MIXER_SCHN_FRONT_LEFT, &m_current_vol) < 0)
            {
                perror("alsa: error getting playback volume");
                m_current_vol = -1;
                return;
            }
            m_muted = m_current_vol == m_min_vol;
        }
    }

    [[nodiscard]] double current_volume_percentage() const
    {
        if (m_min_vol == -1 || m_max_vol == -1 || m_current_vol == -1 || m_max_vol - m_min_vol <= 0)
        {
            return 0.0;
        }
        return 100.0 * (double)(m_current_vol - m_min_vol) / (double)(m_max_vol - m_min_vol);
    }

    [[nodiscard]] bool is_muted() const
    {
        return m_min_vol == -1 || m_max_vol == -1 || m_current_vol == -1 || m_max_vol - m_min_vol <= 0 || m_muted;
    }

    void mute_toggle()
    {
        if (m_min_vol == -1 || m_max_vol == -1 || m_current_vol == -1 || m_max_vol - m_min_vol <= 0)
        {
            return;
        }
        m_before_mute_vol = m_current_vol;
        if (snd_mixer_selem_has_playback_switch(m_elem))
        {
            if (snd_mixer_selem_set_playback_switch_all(m_elem, (int)m_muted) < 0)
            {
                perror("alsa: failed to set playback switch");
                return;
            }
        }
        else
        {
            long new_volume_level = m_muted ? m_before_mute_vol : m_min_vol;
            if (snd_mixer_selem_set_playback_volume_all(m_elem, new_volume_level) < 0)
            {
                perror("alsa: failed to set playback volume");
                return;
            }
            m_before_mute_vol = m_current_vol;
            m_current_vol     = new_volume_level;
        }
        m_muted = !m_muted;
    }

    void decrease_volume()
    {
        if (m_min_vol == -1 || m_max_vol == -1 || m_current_vol == -1 || m_max_vol - m_min_vol <= 0)
        {
            return;
        }

        long   range           = m_max_vol - m_min_vol;
        double current_percent = 100.0 * (double)(m_current_vol - m_min_vol) / (double)range;
        double new_percent     = current_percent - 5.0;
        new_percent            = std::max(0.0, std::min(100.0, new_percent));
        long new_vol           = m_min_vol + lround((new_percent / 100.0) * (double)range);
        if (new_vol == m_current_vol)
        {
            return;
        }

        if (snd_mixer_selem_set_playback_volume_all(m_elem, new_vol) < 0)
        {
            perror("alsa: failed to set playback volume");
            return;
        }

        m_current_vol = new_vol;
    }

    void increase_volume()
    {
        if (m_min_vol == -1 || m_max_vol == -1 || m_current_vol == -1 || m_max_vol - m_min_vol <= 0)
        {
            return;
        }

        long   range           = m_max_vol - m_min_vol;
        double current_percent = 100.0 * (double)(m_current_vol - m_min_vol) / (double)range;
        double new_percent     = current_percent + 5.0;
        new_percent            = std::max(0.0, std::min(100.0, new_percent));
        long new_vol           = m_min_vol + lround((new_percent / 100.0) * (double)range);
        if (new_vol == m_current_vol)
        {
            return;
        }

        if (snd_mixer_selem_set_playback_volume_all(m_elem, new_vol) < 0)
        {
            perror("alsa: failed to set playback volume");
            return;
        }

        m_current_vol = new_vol;
    }

  private:
    snd_mixer_t*          m_mixer           = nullptr;
    snd_mixer_selem_id_t* m_sid             = nullptr;
    snd_mixer_elem_t*     m_elem            = nullptr;
    long                  m_min_vol         = -1;
    long                  m_max_vol         = -1;
    long                  m_current_vol     = -1;
    long                  m_before_mute_vol = -1;
    bool                  m_muted           = false;
};

#endif